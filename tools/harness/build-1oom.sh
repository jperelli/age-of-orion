#!/usr/bin/env bash
# Builds the C reference: 1oom at the commit Age of Orion was ported from, with the small
# harness patch (computer-only games, a save every year, stop after N years).
# Usage: tools/harness/build-1oom.sh [DIR]   (default: target/1oom)
# Needs: git, gcc, make, autoconf, automake, libtool.
set -euo pipefail
cd "$(dirname "$0")/../.."
dir="${1:-target/1oom}"
commit=42e9c0b
if [ ! -d "$dir/.git" ]; then
  git clone --quiet https://github.com/1oom-fork/1oom.git "$dir"
fi
patch="$(pwd)/tools/harness/1oom-harness.patch"
cd "$dir"
git checkout --quiet -f "$commit"
git apply "$patch"
autoreconf -fi >/dev/null 2>&1
./configure --disable-uiclassic --disable-hwsdl1 --disable-hwsdl2 --disable-hwalleg4 --disable-tools >/dev/null
make -j"$(nproc)" >/dev/null
echo "built $dir/src/1oom_cmdline"
