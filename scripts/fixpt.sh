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
(cd "$clone" && cargo build --release --quiet -p fixpt-cli) >&2
echo "$clone/target/release/fixpt"
