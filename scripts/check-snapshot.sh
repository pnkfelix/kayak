#!/bin/sh
# scripts/check.sh on a snapshot of the working tree, in a worktree of its
# own (C277): the tree may be edited while the check runs, and what passed
# is exactly the snapshot. The user's: "make sure your script prints out the
# directory".
#
# usage: scripts/check-snapshot.sh [--quick]     check a snapshot
#        scripts/check-snapshot.sh --same        whether the working tree is
#                                                the snapshot that last passed
#
# The snapshot is every file git would add (`git add -A`, .gitignore
# respected), as a commit on HEAD made through an index of its own: the
# working tree and its index untouched. The worktree is ../kayak-snapshot,
# beside this one (so that ../kayak-refs/fixpt is found from it too), kept
# from run to run so that its target/ builds incrementally; its
# target/fixpt-cache is this tree's, shared. The check's log is kept there,
# check.log. When the check passes, the Fixpt answers it kept or dropped
# (tests/fixpt-answers/) are copied back here, and the snapshot's tree
# recorded for --same.
set -eu
cd "$(dirname "$0")/.."
here=$(pwd)
snap=$(cd .. && pwd)/kayak-snapshot

tree_now() {
    idx=$(mktemp)
    cp "$(git rev-parse --git-dir)/index" "$idx"
    GIT_INDEX_FILE=$idx git add -A
    GIT_INDEX_FILE=$idx git write-tree
    rm -f "$idx"
}

if [ "${1-}" = --same ]; then
    passed=$(cat "$snap/.passed-tree" 2>/dev/null || echo none)
    now=$(tree_now)
    if [ "$passed" = "$now" ]; then
        echo "check-snapshot: the working tree is the snapshot that passed ($now)"
        exit 0
    fi
    echo "check-snapshot: the working tree ($now) is not the snapshot that passed ($passed)"
    exit 1
fi

tree=$(tree_now)
commit=$(git commit-tree "$tree" -p HEAD -m "check-snapshot of $tree")
if [ ! -d "$snap" ]; then
    git worktree add --detach "$snap" "$commit" > /dev/null
fi
git -C "$snap" checkout --quiet --detach --force "$commit"
git -C "$snap" clean --quiet -ffdx -e /target -e /check.log -e /.passed-tree
mkdir -p "$snap/target" target/fixpt-cache
[ -L "$snap/target/fixpt-cache" ] || { rm -rf "$snap/target/fixpt-cache"; ln -s "$here/target/fixpt-cache" "$snap/target/fixpt-cache"; }
rm -f "$snap/.passed-tree"
echo "check-snapshot: directory $snap"
echo "check-snapshot: snapshot $commit (tree $tree)"
echo "check-snapshot: log $snap/check.log"
status=0
(cd "$snap" && scripts/check.sh "$@") > "$snap/check.log" 2>&1 || status=$?
tail -n 3 "$snap/check.log"
if [ $status -ne 0 ] || ! tail -n 1 "$snap/check.log" | grep -q '^check: all passed'; then
    echo "check-snapshot: FAILED (directory $snap, log $snap/check.log)"
    exit 1
fi
# What the check changed of the snapshot's Fixpt answers, brought back.
changed=$(cd "$snap" && git status --porcelain --untracked-files=all -- tests/fixpt-answers | cut -c4-)
for f in $changed; do
    if [ -f "$snap/$f" ]; then cp "$snap/$f" "$here/$f"; else rm -f "$here/$f"; fi
done
[ -z "$changed" ] || echo "check-snapshot: $(echo "$changed" | wc -l | tr -d ' ') Fixpt answer file(s) brought back"
other=$(cd "$snap" && git status --porcelain --untracked-files=all | grep -v ' tests/fixpt-answers/' | grep -v ' check.log$' || true)
[ -z "$other" ] || { echo "check-snapshot: the check also changed, left in $snap:"; echo "$other"; }
echo "$tree" > "$snap/.passed-tree"
echo "check-snapshot: passed (directory $snap)"
