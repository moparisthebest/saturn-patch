/*----------------------------------------------------------------------------*/
/*-- converted from cdrom.inc with https://c2rust.com/                      --*/
/*-- Rebuild CD sector fields                                               --*/
/*-- Copyright (C) 2012 CUE                                                 --*/
/*--                                                                        --*/
/*-- This program is free software: you can redistribute it and/or modify   --*/
/*-- it under the terms of the GNU General Public License as published by   --*/
/*-- the Free Software Foundation, either version 3 of the License, or      --*/
/*-- (at your option) any later version.                                    --*/
/*--                                                                        --*/
/*-- This program is distributed in the hope that it will be useful,        --*/
/*-- but WITHOUT ANY WARRANTY; without even the implied warranty of         --*/
/*-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the           --*/
/*-- GNU General Public License for more details.                           --*/
/*--                                                                        --*/
/*-- You should have received a copy of the GNU General Public License      --*/
/*-- along with this program. If not, see <http://www.gnu.org/licenses/>.   --*/
/*----------------------------------------------------------------------------*/
#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc;

/*----------------------------------------------------------------------------*/

static mut CDROM_crc: [libc::c_uint; 256] = [0; 256];

static mut CDROM_exp: [libc::c_uchar; 256] = [0; 256];

static mut CDROM_log: [libc::c_uchar; 256] = [0; 256];

static mut CDROM_enabled: libc::c_uint = 0 as libc::c_int as libc::c_uint;
/*----------------------------------------------------------------------------*/
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Init_Tables() {
    let mut edc: libc::c_uint = 0;
    let mut ecc: libc::c_uint = 0;
    let mut i: libc::c_uint = 0;
    let mut j: libc::c_uint = 0;
    i = 0 as libc::c_int as libc::c_uint;
    while i < 0x100 as libc::c_int as libc::c_uint {
        edc = i;
        j = 0 as libc::c_int as libc::c_uint;
        while j < 8 as libc::c_int as libc::c_uint {
            edc = if edc & 1 as libc::c_int as libc::c_uint != 0 {
                (edc >> 1 as libc::c_int) ^ 0xd8018001 as libc::c_uint
            } else {
                (edc) >> 1 as libc::c_int
            };
            j = j.wrapping_add(1)
        }
        CDROM_crc[i as usize] = edc;
        ecc = if i & 0x80 as libc::c_int as libc::c_uint != 0 {
            (i << 1 as libc::c_int) ^ 0x11d as libc::c_int as libc::c_uint
        } else {
            (i) << 1 as libc::c_int
        };
        CDROM_exp[i as usize] = ecc as libc::c_uchar;
        CDROM_log[(i ^ ecc) as usize] = i as libc::c_uchar;
        i = i.wrapping_add(1)
    }
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_NUM2BCD(mut value: libc::c_char) -> libc::c_char {
    return ((value as libc::c_int / 10 as libc::c_int) << 4 as libc::c_int | value as libc::c_int % 10 as libc::c_int) as libc::c_char;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_BCD2NUM(mut value: libc::c_char) -> libc::c_char {
    return ((value as libc::c_int >> 4 as libc::c_int) * 10 as libc::c_int | value as libc::c_int & 0xf as libc::c_int) as libc::c_char;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Update(mut sector: *mut libc::c_char, mut lba: libc::c_int, mut cdmode: libc::c_int, mut flags: libc::c_int, mut check: libc::c_int) -> libc::c_int {
    if CDROM_enabled == 0 {
        CDROM_Init_Tables();
        CDROM_enabled = 1 as libc::c_int as libc::c_uint
    }
    if check != 0 && CDROM_Check(sector, cdmode) != 0 {
        return 1 as libc::c_int;
    }
    match cdmode {
        1 => {
            CDROM_Put_Sync(sector.offset(0 as libc::c_int as isize));
            CDROM_Put_Header(sector.offset(0xc as libc::c_int as isize), lba, cdmode);
            CDROM_Put_Intermediate(sector.offset(0x814 as libc::c_int as isize));
            CDROM_Put_EDC(sector.offset(0 as libc::c_int as isize), 0x810 as libc::c_int, sector.offset(0x810 as libc::c_int as isize));
            CDROM_Put_ECC_P(sector.offset(0xc as libc::c_int as isize), sector.offset(0x81c as libc::c_int as isize));
            CDROM_Put_ECC_Q(sector.offset(0xc as libc::c_int as isize), sector.offset(0x8c8 as libc::c_int as isize));
        }
        2 => {
            CDROM_Put_Sync(sector.offset(0 as libc::c_int as isize));
            CDROM_Put_Header(sector.offset(0xc as libc::c_int as isize), lba, cdmode);
        }
        21 => {
            CDROM_Put_Sync(sector.offset(0 as libc::c_int as isize));
            //CDROM_Put_Header(sector + POS_HEADER, lba, cdmode);
            CDROM_Put_SubHeader(sector.offset(0x10 as libc::c_int as isize), flags);
            CDROM_Put_EDC(sector.offset(0x10 as libc::c_int as isize), 0x808 as libc::c_int, sector.offset(0x818 as libc::c_int as isize));
            *(sector.offset(0xc as libc::c_int as isize) as *mut libc::c_uint) = 0 as libc::c_int as libc::c_uint;
            CDROM_Put_ECC_P(sector.offset(0xc as libc::c_int as isize), sector.offset(0x81c as libc::c_int as isize));
            CDROM_Put_ECC_Q(sector.offset(0xc as libc::c_int as isize), sector.offset(0x8c8 as libc::c_int as isize));
            CDROM_Put_Header(sector.offset(0xc as libc::c_int as isize), lba, cdmode);
        }
        22 => {
            CDROM_Put_Sync(sector.offset(0 as libc::c_int as isize));
            CDROM_Put_Header(sector.offset(0xc as libc::c_int as isize), lba, cdmode);
            CDROM_Put_SubHeader(sector.offset(0x10 as libc::c_int as isize), flags);
            CDROM_Put_EDC(sector.offset(0x10 as libc::c_int as isize), 0x91c as libc::c_int, sector.offset(0x92c as libc::c_int as isize));
        }
        _ => {}
    }
    return 0 as libc::c_int;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Check(mut sector: *mut libc::c_char, mut cdmode: libc::c_int) -> libc::c_int {
    let mut tmp1: libc::c_int = 0;
    let mut tmp2: libc::c_int = 0;
    let mut tmp3: libc::c_int = 0;
    match cdmode {
        1 => {
            if *(sector.offset(0 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff00 as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(4 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffffff as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(8 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff as libc::c_int as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *sector.offset(0xf as libc::c_int as isize) as libc::c_int != 1 as libc::c_int {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0x814 as libc::c_int as isize) as *mut libc::c_int) != 0 {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0x814 as libc::c_int as isize).offset(4 as libc::c_int as isize) as *mut libc::c_int) != 0 {
                return 1 as libc::c_int;
            }
        }
        2 => {
            if *(sector.offset(0 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff00 as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(4 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffffff as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(8 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff as libc::c_int as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *sector.offset(0xf as libc::c_int as isize) as libc::c_int != 2 as libc::c_int {
                return 1 as libc::c_int;
            }
        }
        21 => {
            if *(sector.offset(0 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff00 as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(4 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffffff as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(8 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff as libc::c_int as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *sector.offset(0xf as libc::c_int as isize) as libc::c_int != 2 as libc::c_int {
                return 1 as libc::c_int;
            }
            tmp1 = *(sector.offset(0x10 as libc::c_int as isize) as *mut libc::c_int);
            tmp2 = *(sector.offset(0x10 as libc::c_int as isize).offset(4 as libc::c_int as isize) as *mut libc::c_int);
            tmp3 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x20 as libc::c_int;
            if tmp1 == 0 || tmp1 != tmp2 || tmp3 != 0 {
                return 1 as libc::c_int;
            }
            tmp1 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x2 as libc::c_int;
            tmp2 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x4 as libc::c_int;
            tmp3 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x8 as libc::c_int;
            if tmp1 != 0 && (tmp2 != 0 || tmp3 != 0) {
                return 1 as libc::c_int;
            }
            if tmp2 != 0 && (tmp3 != 0 || tmp1 != 0) {
                return 1 as libc::c_int;
            }
            if tmp3 != 0 && (tmp1 != 0 || tmp2 != 0) {
                return 1 as libc::c_int;
            }
        }
        22 => {
            if *(sector.offset(0 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff00 as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(4 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffffff as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *(sector.offset(0 as libc::c_int as isize).offset(8 as libc::c_int as isize) as *mut libc::c_uint) != 0xffffff as libc::c_int as libc::c_uint {
                return 1 as libc::c_int;
            }
            if *sector.offset(0xf as libc::c_int as isize) as libc::c_int != 2 as libc::c_int {
                return 1 as libc::c_int;
            }
            tmp1 = *(sector.offset(0x10 as libc::c_int as isize) as *mut libc::c_int);
            tmp2 = *(sector.offset(0x10 as libc::c_int as isize).offset(4 as libc::c_int as isize) as *mut libc::c_int);
            tmp3 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x20 as libc::c_int;
            if tmp1 == 0 || tmp1 != tmp2 || tmp3 == 0 {
                return 1 as libc::c_int;
            }
            tmp1 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x2 as libc::c_int;
            tmp2 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x4 as libc::c_int;
            tmp3 = *sector.offset(0x12 as libc::c_int as isize) as libc::c_int & 0x8 as libc::c_int;
            if tmp1 != 0 && (tmp2 != 0 || tmp3 != 0) {
                return 1 as libc::c_int;
            }
            if tmp2 != 0 && (tmp3 != 0 || tmp1 != 0) {
                return 1 as libc::c_int;
            }
            if tmp3 != 0 && (tmp1 != 0 || tmp2 != 0) {
                return 1 as libc::c_int;
            }
        }
        _ => {
            return 1 as libc::c_int;
        }
    }
    return 0 as libc::c_int;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Put_Sync(mut source: *mut libc::c_char) {
    *(source as *mut libc::c_uint) = 0xffffff00 as libc::c_uint;
    *(source.offset(4 as libc::c_int as isize) as *mut libc::c_uint) = 0xffffffff as libc::c_uint;
    *(source.offset(8 as libc::c_int as isize) as *mut libc::c_uint) = 0xffffff as libc::c_int as libc::c_uint;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Put_Header(mut source: *mut libc::c_char, mut lba: libc::c_int, mut cdmode: libc::c_int) {
    lba += 150 as libc::c_int;
    *source = CDROM_NUM2BCD((lba / 75 as libc::c_int / 60 as libc::c_int) as libc::c_char);
    *source.offset(1 as libc::c_int as isize) = CDROM_NUM2BCD((lba / 75 as libc::c_int % 60 as libc::c_int) as libc::c_char);
    *source.offset(2 as libc::c_int as isize) = CDROM_NUM2BCD((lba % 75 as libc::c_int) as libc::c_char);
    *source.offset(3 as libc::c_int as isize) = cdmode as libc::c_char;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Put_SubHeader(mut source: *mut libc::c_char, mut flags: libc::c_int) {
    *(source as *mut libc::c_uint) = flags as libc::c_uint;
    *(source.offset(4 as libc::c_int as isize) as *mut libc::c_uint) = flags as libc::c_uint;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Put_Intermediate(mut source: *mut libc::c_char) {
    *(source as *mut libc::c_uint) = 0 as libc::c_int as libc::c_uint;
    *(source.offset(4 as libc::c_int as isize) as *mut libc::c_uint) = 0 as libc::c_int as libc::c_uint;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Put_EDC(mut source: *mut libc::c_char, mut length: libc::c_int, mut target: *mut libc::c_char) {
    let mut edc: libc::c_uint = 0;
    let mut i: libc::c_uint = 0;
    edc = 0 as libc::c_int as libc::c_uint;
    i = 0 as libc::c_int as libc::c_uint;
    while i < length as libc::c_uint {
        edc = edc >> 8 as libc::c_int ^ CDROM_crc[((edc ^ *source.offset(i as isize) as libc::c_uint) & 0xff as libc::c_int as libc::c_uint) as usize];
        i = i.wrapping_add(1)
    }
    *(target as *mut libc::c_uint) = edc;
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Put_ECC_P(mut source: *mut libc::c_char, mut target: *mut libc::c_char) {
    let mut table: libc::c_uint = 0;
    let mut column: libc::c_uint = 0;
    let mut row: libc::c_uint = 0;
    let mut index: libc::c_uint = 0;
    let mut ecc: libc::c_uchar = 0;
    let mut xor: libc::c_uchar = 0;
    table = 0 as libc::c_int as libc::c_uint;
    while table < 2 as libc::c_int as libc::c_uint {
        column = 0 as libc::c_int as libc::c_uint;
        while column < 43 as libc::c_int as libc::c_uint {
            xor = 0 as libc::c_int as libc::c_uchar;
            ecc = xor;
            row = 0 as libc::c_int as libc::c_uint;
            while row < 24 as libc::c_int as libc::c_uint {
                index = (1 as libc::c_int as libc::c_uint)
                    .wrapping_mul(column)
                    .wrapping_add((43 as libc::c_int as libc::c_uint).wrapping_mul(row))
                    .wrapping_rem((24 as libc::c_int * 43 as libc::c_int) as libc::c_uint);
                ecc = (ecc as libc::c_int ^ *source.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(index).wrapping_add(table) as isize) as libc::c_int) as libc::c_uchar;
                ecc = CDROM_exp[ecc as usize];
                xor = (xor as libc::c_int ^ *source.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(index).wrapping_add(table) as isize) as libc::c_int) as libc::c_uchar;
                row = row.wrapping_add(1)
            }
            ecc = CDROM_log[(CDROM_exp[ecc as usize] as libc::c_int ^ xor as libc::c_int) as usize];
            *target.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(column).wrapping_add(table) as isize) = ecc as libc::c_char;
            *target.offset(
                (2 as libc::c_int as libc::c_uint)
                    .wrapping_mul(column.wrapping_add(43 as libc::c_int as libc::c_uint))
                    .wrapping_add(table) as isize,
            ) = (ecc as libc::c_int ^ xor as libc::c_int) as libc::c_char;
            column = column.wrapping_add(1)
        }
        table = table.wrapping_add(1)
    }
}
/*----------------------------------------------------------------------------*/

unsafe fn CDROM_Put_ECC_Q(mut source: *mut libc::c_char, mut target: *mut libc::c_char) {
    let mut table: libc::c_uint = 0;
    let mut row: libc::c_uint = 0;
    let mut column: libc::c_uint = 0;
    let mut index: libc::c_uint = 0;
    let mut ecc: libc::c_uchar = 0;
    let mut xor: libc::c_uchar = 0;
    table = 0 as libc::c_int as libc::c_uint;
    while table < 2 as libc::c_int as libc::c_uint {
        row = 0 as libc::c_int as libc::c_uint;
        while row < 26 as libc::c_int as libc::c_uint {
            xor = 0 as libc::c_int as libc::c_uchar;
            ecc = xor;
            column = 0 as libc::c_int as libc::c_uint;
            while column < 43 as libc::c_int as libc::c_uint {
                index = (43 as libc::c_int as libc::c_uint)
                    .wrapping_mul(row)
                    .wrapping_add(((1 as libc::c_int + 43 as libc::c_int) as libc::c_uint).wrapping_mul(column))
                    .wrapping_rem((26 as libc::c_int * 43 as libc::c_int) as libc::c_uint);
                ecc = (ecc as libc::c_int ^ *source.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(index).wrapping_add(table) as isize) as libc::c_int) as libc::c_uchar;
                ecc = CDROM_exp[ecc as usize];
                xor = (xor as libc::c_int ^ *source.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(index).wrapping_add(table) as isize) as libc::c_int) as libc::c_uchar;
                column = column.wrapping_add(1)
            }
            ecc = CDROM_log[(CDROM_exp[ecc as usize] as libc::c_int ^ xor as libc::c_int) as usize];
            *target.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(row).wrapping_add(table) as isize) = ecc as libc::c_char;
            *target.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(row.wrapping_add(26 as libc::c_int as libc::c_uint)).wrapping_add(table) as isize) =
                (ecc as libc::c_int ^ xor as libc::c_int) as libc::c_char;
            row = row.wrapping_add(1)
        }
        table = table.wrapping_add(1)
    }
}

unsafe fn Check_Image(mut sector: &[u8], mut length: *mut libc::c_int, mut offset: *mut libc::c_int, mut position: *mut libc::c_int) -> libc::c_int {
    let mut buffer: [u8; 8192] = [0; 8192];
    let mut i: libc::c_int = 0;
    let mut n: libc::c_int = 0;
    let mut tabla: [[libc::c_int; 3]; 8] = [
        [0 as libc::c_int, 0x800 as libc::c_int, 0 as libc::c_int],
        [1 as libc::c_int, 0x930 as libc::c_int, 0x10 as libc::c_int],
        [21 as libc::c_int, 0x930 as libc::c_int, 0x18 as libc::c_int],
        [1 as libc::c_int, 0x990 as libc::c_int, 0x10 as libc::c_int],
        [21 as libc::c_int, 0x990 as libc::c_int, 0x18 as libc::c_int],
        [1 as libc::c_int, 0x940 as libc::c_int, 0x10 as libc::c_int],
        [21 as libc::c_int, 0x940 as libc::c_int, 0x18 as libc::c_int],
        [-(1 as libc::c_int), -(1 as libc::c_int), -(1 as libc::c_int)],
    ];
    n = 0 as libc::c_int;
    while n < 2 as libc::c_int {
        i = 0 as libc::c_int;
        while tabla[i as usize][0 as libc::c_int as usize] != -(1 as libc::c_int) {
            *length = tabla[i as usize][1 as libc::c_int as usize];
            *position = tabla[i as usize][2 as libc::c_int as usize];
            loop {
                *offset = *length * 150 as libc::c_int * n;
                //fseek(fp, *offset + 0x000010 * *length, SEEK_SET);
                //if (fread(buffer, 2, *length, fp) != *length) return -1;
                // manually implement above fseek+fread
                {
                    let copy_start = (*offset + 0x000010 * *length) as usize;
                    let copy_end = copy_start + (2 * *length) as usize;
                    if copy_start >= sector.len() || copy_end >= sector.len() {
                        return -1;
                    }
                    let src = &sector[copy_start..copy_end];
                    &buffer[0..src.len()].copy_from_slice(src);
                }

                if *(buffer.as_mut_ptr().offset(*position as isize) as *mut libc::c_int) == 0x30444301 as libc::c_int {
                    if *(buffer.as_mut_ptr().offset(*position as isize).offset(4 as libc::c_int as isize) as *mut libc::c_int) == 0x13130 as libc::c_int {
                        if *(buffer.as_mut_ptr().offset(*position as isize).offset(*length as isize).offset(1 as libc::c_int as isize) as *mut libc::c_int) == 0x30304443 as libc::c_int {
                            return tabla[i as usize][0 as libc::c_int as usize];
                        }
                    }
                }
                if *position < 0x10 as libc::c_int {
                    break;
                }
                *length -= 0x10 as libc::c_int;
                *position -= 0x10 as libc::c_int
            }
            i += 1
        }
        n += 1
    }
    return -(1 as libc::c_int);
}
/*----------------------------------------------------------------------------*/
/*--  EOF                                           Copyright (C) 2012 CUE  --*/
/*----------------------------------------------------------------------------*/
/*----------------------------------------------------------------------------*/

// above as you can see was roughly and unsafely converted from C, until I get around to cleaning that
// mess up, write some slightly nicer and rustier APIs to use

use anyhow::{bail, Result};

pub struct CDRomImage {
    mode: i32,
    length: usize,
    offset: usize,
    position: usize,
}

impl CDRomImage {
    pub fn new(bytes: &[u8]) -> Result<CDRomImage> {
        let mode: i32;
        let mut length = 0;
        let mut offset = 0;
        let mut position = 0;

        unsafe {
            mode = Check_Image(&bytes, &mut length, &mut offset, &mut position);
        }
        if mode == -1 {
            bail!("File not supported or invalid");
        }
        Ok(CDRomImage {
            mode,
            length: length as usize,
            offset: offset as usize,
            position: position as usize,
        })
    }

    pub fn update_sectors(&self, bytes: &mut [u8]) {
        unsafe {
            CDROM_Update(bytes.as_mut_ptr() as *mut libc::c_char, 0, self.mode, 0, 0);
            CDROM_Update(bytes[self.length..].as_mut_ptr() as *mut libc::c_char, 1, self.mode, 0, 0);
        }
    }

    pub fn debug(&self) {
        println!("mode: {}, length: {}, offset: {}, position: {}", self.mode, self.length, self.offset, self.position);
    }
}
