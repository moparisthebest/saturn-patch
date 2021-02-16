use std::cmp::min;
use std::convert::TryInto;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{bail, Result};

use hmac_sha256::Hash;

mod cdrom;

use crate::cdrom::CDRomImage;

// you may change this if you wish
pub const DESIRED_SATURN_DISC: SaturnDisc = SaturnDisc {
    // desired regions in order of preference (some disks only support 1 region, in which case first would be picked)
    desired_region_bytes: *br"JUBLKTEA",
    // will replace manufacturer bytes, so you can boot with KD02 black boot disc, requires 16 bytes exactly
    desired_mfr_bytes: *br"SEGA TP T-81    ",
    // this is what https://madroms.satakore.com/ uses
    //desired_mfr_bytes: *br"SEGA TP ERPRISES"
};

// don't change below here
const SEGA_SATURN_BYTES: &[u8; 16] = br"SEGA SEGASATURN ";

const BACKUP_FILE_EXT: &str = ".saturnpatchbak";

// the order here has to match the order in REGION_STRINGS
const REGION_CODES: &[u8; 8] = br"JTUBKAEL";

const REGION_STRING_LEN: usize = 32;

const REGION_STRINGS: &[&[u8; REGION_STRING_LEN]] = &[
    b"\xA0\x0E\x00\x09For JAPAN.                  ",
    b"\xA0\x0E\x00\x09For TAIWAN and PHILIPINES.  ",
    b"\xA0\x0E\x00\x09For USA and CANADA.         ",
    b"\xA0\x0E\x00\x09For BRAZIL.                 ",
    b"\xA0\x0E\x00\x09For KOREA.                  ",
    b"\xA0\x0E\x00\x09For ASIA PAL area.          ",
    b"\xA0\x0E\x00\x09For EUROPE.                 ",
    b"\xA0\x0E\x00\x09For LATIN AMERICA.          ",
];

// 1 byte version, 32 byte header hash, and more bytes after but that's verified with a hash so it's fine
const MIN_BACKUP_SIZE: usize = 1 + 32 + 1;

fn region_index(region: &u8) -> Result<usize> {
    let mut i = 0;
    for c in REGION_CODES {
        if c == region {
            return Ok(i);
        }
        i += 1;
    }
    bail!("invalid region: {}", *region as char);
}

fn region_copy_sort_pad(regions: &[u8]) -> Vec<u8> {
    let mut ret = regions.to_vec();
    ret.sort_by(|a, b| region_index(a).unwrap_or(10).cmp(&region_index(b).unwrap_or(10)));
    while ret.len() < 16 {
        ret.push(b' ');
    }
    ret
}

fn region_count(regions: &[u8]) -> usize {
    let mut ret = 0;
    while ret < regions.len() {
        if regions[ret] == b' ' {
            return ret;
        }
        ret += 1;
    }
    ret
}

fn first_index_of(file_name: &OsString, haystack: &[u8], needle: &[u8]) -> Result<usize> {
    for i in 0..haystack.len() - needle.len() + 1 {
        if haystack[i..i + needle.len()] == needle[..] {
            return Ok(i);
        }
    }
    bail!("not saturn image? {:?}", file_name);
}

pub struct SaturnDisc {
    pub desired_region_bytes: [u8; 8],
    pub desired_mfr_bytes: [u8; 16],
}

impl SaturnDisc {
    pub fn from_env_args() -> Result<SaturnDisc> {
        let desired_mfr_bytes = match std::env::var("SATURN_MANUFACTURER") {
            Ok(k) => {
                let mfr_bytes = k.as_bytes();
                if mfr_bytes.len() == 0 {
                    DESIRED_SATURN_DISC.desired_mfr_bytes
                } else {
                    if mfr_bytes.len() > 16 {
                        bail!("SATURN_MANUFACTURER length {} exceeds max length of 16", mfr_bytes.len());
                    }
                    let mut ret = [20u8; 16];
                    ret[0..mfr_bytes.len()].copy_from_slice(mfr_bytes);
                    ret
                }
            }
            Err(_) => DESIRED_SATURN_DISC.desired_mfr_bytes,
        };

        let desired_region_bytes = match std::env::var("SATURN_REGION") {
            Ok(k) => {
                let k = k.to_ascii_uppercase();
                let region_bytes = k.as_bytes();
                if region_bytes.len() == 0 {
                    DESIRED_SATURN_DISC.desired_region_bytes
                } else {
                    if region_bytes.len() > 8 {
                        bail!("SATURN_REGION length {} exceeds max length of 8", region_bytes.len());
                    }
                    // validate each character is a supported region
                    for region in region_bytes {
                        region_index(region)?;
                    }
                    {
                        // ensure no duplicates exist
                        let mut region_vec = region_bytes.to_vec();
                        region_vec.sort();
                        region_vec.dedup();
                        if region_bytes.len() != region_bytes.len() {
                            bail!("SATURN_REGION must not have duplicate regions");
                        }
                    }
                    let mut empty = [20u8; 8];
                    empty[0..region_bytes.len()].copy_from_slice(region_bytes);
                    if region_bytes.len() != 8 {
                        // pad it out, we always want all possible regions so we can always overwrite entire region string
                        let mut x = region_bytes.len();
                        REGION_CODES.iter().filter(|r| !region_bytes.contains(r)).for_each(|r| {
                            empty[x] = *r;
                            x += 1;
                        });
                    }
                    println!("SATURN_REGION: {}", String::from_utf8_lossy(&empty));
                    empty
                }
            }
            Err(_) => DESIRED_SATURN_DISC.desired_region_bytes,
        };

        Ok(SaturnDisc {
            desired_region_bytes,
            desired_mfr_bytes,
        })
    }

    pub fn patch(&self, file_name: &OsString) -> Result<()> {
        let path = Path::new(file_name);
        if !path.is_file() {
            bail!("file does not exist: {:?}", file_name);
        }
        let mut file = File::open(file_name)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // only look in first 256 bytes
        let header_offset = first_index_of(file_name, &bytes[0..min(bytes.len(), 256)], SEGA_SATURN_BYTES)?;

        let cdrom = CDRomImage::new(&bytes)?;

        let orig_hash = Hash::hash(&bytes);
        // now update the sectors which shouldn't do anything, and ensure it didn't do anything, if it did, bail
        cdrom.update_sectors(&mut bytes);
        if orig_hash != Hash::hash(&bytes) {
            bail!("existing CD has bad sectors? refusing to update because won't be able to restore original file: {:?}", file_name);
        }

        let change_mfr = self.desired_mfr_bytes != &bytes[(header_offset + 16)..(header_offset + 32)];
        let region_bytes = &bytes[(header_offset + 64)..(header_offset + 80)];
        let region_count = region_count(&region_bytes);
        let new_region = region_copy_sort_pad(&self.desired_region_bytes[0..region_count]);
        let change_region = new_region != region_bytes;
        // copy this for use later replacing strings
        let region_bytes = &region_bytes[0..region_count].to_vec();

        if change_mfr || change_region {
            // first we need to find first region string index
            let mut first_region_string_index = bytes.len();
            let mut region_string_indices = Vec::with_capacity(region_count);
            let mut string_begin = header_offset + 80 + 16; // end of header
            let mut string_end = bytes.len();
            // replace strings
            for i in 0..region_count {
                // these do NOT appear to be in the same order as the characters in the header, so just look anywhere, I guess...
                let region_string = REGION_STRINGS[region_index(&region_bytes[i])?];
                // have to add string_begin back in because first_index_of returns based on 0, and we are sending in a slice
                let string_offset = first_index_of(file_name, &bytes[string_begin..string_end], region_string)? + string_begin;
                if i == 0 && region_count > 1 {
                    // calculate max/min for future searches
                    // we might have found the last, so first would be this far back exactly
                    string_begin = string_offset - ((region_count - 1) * REGION_STRING_LEN);
                    // and if we found the first, then we might be twice the length back
                    string_end = string_begin + (region_count * REGION_STRING_LEN * 2);
                }
                if string_offset < first_region_string_index {
                    first_region_string_index = string_offset;
                }
                region_string_indices.push(string_offset);
            }

            let mut backup_file_path = file_name.clone();
            backup_file_path.push(BACKUP_FILE_EXT);
            let backup_file_path = Path::new(&backup_file_path);
            let write_header = !backup_file_path.is_file();
            let mut backup_vec = Vec::new();
            if write_header {
                // only write a header backup if one doesn't exist
                // first write original file sha256 hash, exactly 32 bytes
                backup_vec.write_all(&orig_hash)?;
                // next write region_count as a u8
                backup_vec.push(region_count as u8);
                // next write first_region_string_index as a u32 in be/network byte order
                backup_vec.write_all(&(first_region_string_index as u32).to_be_bytes())?;
                // next write original manufacturer, always 16 bytes
                backup_vec.write_all(&bytes[(header_offset + 16)..(header_offset + 32)])?;
                // next write original regions, always 16 bytes
                backup_vec.write_all(&bytes[(header_offset + 64)..(header_offset + 80)])?;
                // next write original region strings, length depends on region_count
                backup_vec.write_all(&bytes[first_region_string_index..(first_region_string_index + (region_count * REGION_STRING_LEN))])?;
            }

            if change_mfr {
                &bytes[(header_offset + 16)..(header_offset + 32)].copy_from_slice(&self.desired_mfr_bytes);
            }

            if change_region {
                &bytes[(header_offset + 64)..(header_offset + 80)].copy_from_slice(&new_region);
                // this way does it in order vs using region_string_indices which replaces them in the order
                // the disc already had them in, is one more right than the other?
                //let mut string_offset = first_region_string_index;
                // replace strings
                for i in 0..region_count {
                    let new_region_string = REGION_STRINGS[region_index(&new_region[i])?];
                    let string_offset = region_string_indices[i];
                    &bytes[string_offset..(string_offset + REGION_STRING_LEN)].copy_from_slice(new_region_string);
                    //string_offset += REGION_STRING_LEN;
                }
            }

            cdrom.update_sectors(&mut bytes);

            if write_header {
                // only write a header backup if one doesn't exist
                let mut backup_file = File::create(backup_file_path)?;
                // first write 1 byte for a version number in case this format changes, 0 for now
                backup_file.write_all(&[0])?;
                // next write the sha256 hash of the rest of this backup file so we can verify it's good when we read it in, exactly 32 bytes
                backup_file.write_all(&Hash::hash(&backup_vec))?;
                // then write the rest of the file
                backup_file.write_all(&backup_vec)?;
                drop(backup_vec); // we don't need this anymore
            }

            let mut file = File::create(file_name)?;
            file.write_all(&bytes)?;

            print!("SUCCESS: ");
            if write_header {
                print!("wrote header backup, ");
            }
            if change_mfr {
                print!("changed manufacturer, ");
            }
            if change_region {
                print!("changed regions, ");
            }

            println!("patched: {:?}", file_name);
        } else {
            println!("SUCCESS: already desired manufacturer and regions {:?}", file_name);
        }

        Ok(())
    }

    pub fn unpatch(file_name: &OsString) -> Result<()> {
        let path = Path::new(file_name);
        if !path.is_file() {
            bail!("file does not exist: {:?}", file_name);
        }

        let (file_name, header_backup_str) = if path.extension().map_or(false, |ext| ext.eq(&BACKUP_FILE_EXT[1..])) {
            // let's support sending in this too, in which case we'll patch the corresponding non-backup file
            let mut new_path = path.to_path_buf();
            new_path.set_file_name(path.file_stem().expect("no file name? impossible without extension I think..."));
            (new_path.into_os_string(), file_name.to_owned())
        } else {
            let mut header_backup_str = file_name.clone();
            header_backup_str.push(BACKUP_FILE_EXT);
            (file_name.to_owned(), header_backup_str)
        };

        let path = Path::new(&file_name);
        if !path.is_file() {
            bail!("file {:?} does not exist for backup file {:?}", file_name, header_backup_str);
        }

        let header_backup = Path::new(&header_backup_str);
        if !header_backup.is_file() {
            bail!("backup file {:?} does not exist for file {:?}", header_backup_str, file_name);
        }

        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let mut header_backup = File::open(header_backup)?;
        let mut header_bytes = Vec::new();
        header_backup.read_to_end(&mut header_bytes)?;

        if header_bytes.len() < MIN_BACKUP_SIZE {
            bail!(
                "backup file {:?} length {} is less than minimum length of {} for file {:?}",
                header_backup_str,
                header_bytes.len(),
                MIN_BACKUP_SIZE,
                file_name
            );
        }
        if header_bytes[0] != 0 {
            bail!("corrupt backup file or unsupported version {}, only version 0 supported: {:?}", header_bytes[0], header_backup_str);
        }
        if header_bytes[1..33] != Hash::hash(&header_bytes[33..]) {
            bail!("corrupt backup file, hash mismatch: {:?}", header_backup_str);
        }
        // let's cut the cruft we won't need off header_bytes for easier math
        let header_bytes = &header_bytes[33..];

        // only look in first 256 bytes
        let header_offset = first_index_of(&file_name, &bytes[0..min(bytes.len(), 256)], SEGA_SATURN_BYTES)?;

        let cdrom = CDRomImage::new(&bytes)?;

        // let's slice up some views into header_bytes
        let orig_hash = &header_bytes[0..32];
        let region_count = header_bytes[32] as usize;
        let first_region_string_index = u32::from_be_bytes(header_bytes[33..37].try_into()?) as usize;
        // and again to make math more comfortable
        let header_bytes = &header_bytes[37..];
        let manufacturer = &header_bytes[0..16];
        let regions = &header_bytes[16..32];
        let region_strings = &header_bytes[32..];

        if &bytes[(header_offset + 16)..(header_offset + 32)] != manufacturer
            || &bytes[(header_offset + 64)..(header_offset + 80)] != regions
            || &bytes[first_region_string_index..(first_region_string_index + (region_count * REGION_STRING_LEN))] != region_strings
        {
            &bytes[(header_offset + 16)..(header_offset + 32)].copy_from_slice(manufacturer);
            &bytes[(header_offset + 64)..(header_offset + 80)].copy_from_slice(regions);
            &bytes[first_region_string_index..(first_region_string_index + (region_count * REGION_STRING_LEN))].copy_from_slice(region_strings);

            cdrom.update_sectors(&mut bytes);

            if orig_hash != Hash::hash(&bytes) {
                bail!("restore failed, hash mismatch: {:?}", file_name);
            }

            let mut file = File::create(&file_name)?;
            file.write_all(&bytes)?;
            file.flush()?;
            drop(file);

            // we've successfully written the restored file to disk, might as well delete the backup file
            // but we don't want to fail based on this so ignore any errors
            std::fs::remove_file(header_backup_str).ok();

            println!("SUCCESS: unpatched: {:?}", file_name);
        } else if orig_hash == Hash::hash(&bytes) {
            // let's go ahead and try to delete un-needed backup file, but again, ignore errors
            std::fs::remove_file(header_backup_str).ok();

            println!("SUCCESS: already unpatched: {:?}", file_name);
        } else {
            bail!("restore failed, unknown problem: {:?}", file_name);
        }

        Ok(())
    }
}
