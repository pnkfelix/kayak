#!/bin/sh
# Prints the path of the Fixpt binary built from the pinned commit,
# FIXPT_REV, building it first if need be (PLAN.md, References).
#
# The clone is ../kayak-refs/fixpt, or $KAYAK_FIXPT_CLONE. Its HEAD must be
# FIXPT_REV: this script never checks out on its own, since moving the pin
# is a decision, made by editing FIXPT_REV and checking the clone out.
# Exits 3, saying why, when there is no clone.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
clone=${KAYAK_FIXPT_CLONE:-$root/../kayak-refs/fixpt}
want=$(cat "$root/FIXPT_REV")

if [ ! -d "$clone/.git" ]; then
    echo "fixpt.sh: no Fixpt clone at $clone" >&2
    exit 3
fi
have=$(git -C "$clone" rev-parse HEAD)
if [ "$have" != "$want" ]; then
    echo "fixpt.sh: the clone at $clone is at $have, and FIXPT_REV is $want" >&2
    echo "fixpt.sh: check it out (git -C $clone checkout $want), or move the pin" >&2
    exit 1
fi
if [ -n "$(git -C "$clone" status --porcelain --untracked-files=no)" ]; then
    echo "fixpt.sh: the clone at $clone has changes of its own" >&2
    exit 1
fi
# Run from a copy of it in kayak's target/, built and copied under a lock,
# the copy replaced only by a rename: cargo's build, even of nothing,
# removes and links again target/release/fixpt, so that a run of it beside
# another fixpt.sh (check.sh's stages at once) could find it missing.
mkdir -p "$root/target/fixpt"
bin=$root/target/fixpt/fixpt-$want
perl -MFcntl=:flock -e '$^F = 255; open(my $l, ">", shift) or die "$!\n";
    flock($l, LOCK_EX) or die "$!\n"; exec @ARGV or die "$!\n"' \
    "$root/target/fixpt/lock" sh -c '
    set -e
    cd "$1" && cargo build --release --quiet -p fixpt-cli
    if ! cmp -s target/release/fixpt "$2"; then
        cp -p target/release/fixpt "$2.tmp"
        mv -f "$2.tmp" "$2"
    fi' fixpt.sh "$clone" "$bin" >&2
echo "$bin"
