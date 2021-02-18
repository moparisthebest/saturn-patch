# saturn-patch

Reversible region and manufacturer patcher for sega saturn games.

Unlike other utilities that do this, this is open source, safe, cross platform, but most importantly, allows you to
"unpatch" your changed files by storing small backup files next to them.  The sha256 hash of the original file and the
backup file are stored in the backup so you can be confident your games can be put back to byte-for-byte original
whenever you wish.

### How to use

On Windows or some DEs, simply drag saturn bin files onto the saturn-patch executable.  It will only modify saturn files
with the region header in them and then only if it can successfully unpatch it and after it writes the backup file to
disk, so you can safely just run it on all your files.

##### Customize Regions

Set the environmental variable `SATURN_REGION` with your desired regions in order from most preferred to least.  If not
set the default is `JUBLKTEA`, which is Japan, USA, Brazil, Latin America, Taiwan, Europe, Asia.

##### Customize Manufacturer

Set the environmental variable `SATURN_MANUFACTURER` to whatever you wish, as long as it's less than 16 characters.  If not
set the default is `SEGA TP T-81`.  Why would you want to do this?  So you can [swap disks](https://youtu.be/fp-U-s8Xdo0)
with a burned KD02 disk the easy way.

#### How do I get it

You can download `saturn-patch` from the releases section, compile it yourself, or run `cargo install saturn-patch`.
Where is `saturn-unpatch` you ask?  It's the same executable, make a symbolic link to it, or copy and rename.

### Examples

```sh
# patch single file
saturn-patch panzer-dragoon.bin

# unpatch single file
saturn-unpatch panzer-dragoon.bin
# unpatch single file by sending in backup file
saturn-unpatch panzer-dragoon.bin.saturnpatchbak

# patch all bin files recursively in this directory
find -type f -iname '*.bin' -print0 | xargs -0 saturn-patch

# unpatch all bin files for which we have a .saturnpatchbak file for recursively in this directory
find -type f -name '*.saturnpatchbak' -print0 | xargs -0 saturn-unpatch
```
