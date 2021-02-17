#!/bin/bash
set -exo pipefail

echo "starting build for TARGET $TARGET"

export CRATE_NAME=saturn-patch

# cross doesn't actually support stdin/stdout pipes for some reason, skip it for now
DISABLE_TESTS=1

SUFFIX=""

echo "$TARGET" | grep -E '^x86_64-pc-windows-gnu$' >/dev/null && SUFFIX=".exe"

cross rustc --bin saturn-patch --target $TARGET --release

# to check how they are built
file "target/$TARGET/release/saturn-patch$SUFFIX"

# if this commit has a tag, upload artifact to release
strip "target/$TARGET/release/saturn-patch$SUFFIX" || true # if strip fails, it's fine
mkdir -p release
mv "target/$TARGET/release/saturn-patch$SUFFIX" "release/saturn-patch-$TARGET$SUFFIX"

echo 'build success!'
exit 0
