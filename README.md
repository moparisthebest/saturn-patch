saturn-patch
------------

Reversible region and manufacturer patcher for sega saturn games.

Unlike other utilities that do this, this is open source, safe, cross platform, but most importantly, allows you to
"unpatch" your changed files by storing small backup files.  The sha256 hash of the original file and the backup file are
stored in the backup so you can be confident your games can be put back to byte-for-byte original whenever you wish.

Examples
--------

```sh
# patch all bin files recursively in this directory
find -type f -name '*.bin' -print0 | xargs -0 saturn-patch 2>&1 | tee saturn-patch.log
# unpatch all bin files for which we have a .saturnpatchbak file for recursively in this directory
find -type f -name '*.saturnpatchbak' -print0 | xargs -0 saturn-unpatch 2>&1 | tee saturn-unpatch.log
```
