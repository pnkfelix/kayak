#!/usr/bin/env bash
# A PreToolUse hook on Bash (.claude/settings.json): before a command that
# runs one of kayak's timed steps, say how long it is expected to take,
# from `scripts/timed --expect NAME`, to the user (systemMessage) and to
# Claude (additionalContext), whether the command runs in the foreground or
# the background. The step's NAME is the one the script times itself as
# (check-snapshot.sh is `full-check`, not `check-snapshot`). Never blocks:
# any trouble here, and the command runs as it would have.
set -u
root=$(cd "$(dirname "$0")/../.." && pwd)
cmd=$(jq -r '.tool_input.command // ""' 2>/dev/null) || exit 0
[ -n "$cmd" ] || exit 0

names=()
add() { case " ${names[*]-} " in *" $1 "*) ;; *) names+=("$1") ;; esac; }

# Each script that times itself, by the name it uses.
if [[ $cmd =~ check-snapshot\.sh ]]; then
    # `--commit` reruns nothing long; `--quick` is the quick check.
    if [[ $cmd =~ check-snapshot\.sh[^\;\&\|]*--quick ]]; then add quick-check
    elif ! [[ $cmd =~ check-snapshot\.sh[^\;\&\|]*--commit ]]; then add full-check; fi
fi
[[ $cmd =~ pre-check\.sh ]] && add pre-check
[[ $cmd =~ gc-stress\.sh ]] && add gc-stress
if [[ $cmd =~ regen\.sh([^\;\&\|]*) ]]; then
    a=${BASH_REMATCH[1]} n=regen
    [[ $a =~ --all ]] && n=$n-all
    [[ $a =~ --check ]] && n=$n-check
    add "$n"
fi
# `scripts/timed [--at-least N] NAME …`, not `--expect` (already asked).
rest=$cmd
while [[ $rest =~ scripts/timed[[:space:]]+(--at-least[[:space:]]+[0-9]+[[:space:]]+)?([A-Za-z0-9_.-]+)(.*) ]]; do
    n=${BASH_REMATCH[2]} rest=${BASH_REMATCH[3]}
    [ "$n" = "--expect" ] || add "$n"
done

[ "${#names[@]}" -gt 0 ] || exit 0
lines=""
for n in "${names[@]}"; do
    l=$("$root/scripts/timed" --expect "$n" 2>/dev/null | head -1)
    # `--expect` says when it "started"; nothing has, so say none of it.
    l=${l%%; started*}
    [ -n "$l" ] && lines+="$l"$'\n'
done
[ -n "$lines" ] || exit 0
lines=${lines%$'\n'}
jq -n --arg m "$lines" '{
  systemMessage: $m,
  hookSpecificOutput: {
    hookEventName: "PreToolUse",
    additionalContext: ("Expected times of the steps this command runs (scripts/timed): " + $m + ". Say them to the user in your message now, before or as you start this; a step with no runs recorded may be misnamed.")
  }
}'
