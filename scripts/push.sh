#!/bin/sh
# main pushed to GitHub, only if its code passed the full check with every
# job run (C553; the user: "Requiring `--all` before a push sounds
# prudent"): `scripts/check-snapshot.sh --all` records each tree it passes
# (target/check-all-passed, by scripts/tree-digest.sh), and HEAD's must be
# one of them. Else nothing is pushed.
#
# usage: scripts/push.sh
set -eu
cd "$(dirname "$0")/.."
[ $# = 0 ] || { echo "usage: scripts/push.sh" >&2; exit 2; }
[ -z "$(git status --porcelain --untracked-files=no)" ] || { echo "push: the working tree has changes not committed" >&2; exit 1; }
d=$(scripts/tree-digest.sh HEAD)
if ! grep -qx "$d" target/check-all-passed 2>/dev/null; then
    echo "push: HEAD ($(git log -1 --format=%h)) has not passed scripts/check-snapshot.sh --all; nothing pushed" >&2
    exit 1
fi
git fetch -q origin
git merge-base --is-ancestor origin/main HEAD || { echo "push: origin/main is not behind HEAD (not a fast-forward); nothing pushed" >&2; exit 1; }
git push -q origin HEAD:main
echo "push: $(git log -1 --format='%h %s') pushed, $(git rev-list --count HEAD) commits on origin/main"
