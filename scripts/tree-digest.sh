#!/bin/sh
# A tree's code, as `--all` passes it and a push checks it (C553): every
# file of TREE (a tree or commit) by mode, object and path, but those a
# commit's amend after its check changes (CLAUDES_CHOICES.md, the check's
# line; tests/fixpt-answers/, what the check brought back); its SHA-256.
#
# usage: scripts/tree-digest.sh TREE
set -eu
[ $# = 1 ] || { echo "usage: scripts/tree-digest.sh TREE" >&2; exit 2; }
git ls-tree -r --full-tree "$1" | grep -v '	CLAUDES_CHOICES\.md$' | grep -v '	tests/fixpt-answers/' | sha256sum | cut -d' ' -f1
