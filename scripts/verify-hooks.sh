#! /usr/bin/env bash
#
# verify-hooks.sh — check that git hooks are correctly installed
#
# Exits 0 when all hooks exist, are executable, and core.hooksPath
# points to the git-hooks directory.

set -eu

HOOK_DIR='git-hooks'
HOOKS=(commit-msg pre-commit pre-push)
REPO_TOP="$(cd "$(dirname "$0")/.." && pwd)"

all_ok=0

# Check core.hooksPath.
current_hooks="$(git config --local core.hooksPath 2>/dev/null || true)"
if [ "$current_hooks" != "$REPO_TOP/$HOOK_DIR" ]; then
	printf 'ERROR: core.hooksPath is "%s" — expected "%s"\n' "${current_hooks:-<unset>}" "$REPO_TOP/$HOOK_DIR" >&2
	all_ok=1
fi

# Check each hook exists and is executable.
for hook in "${HOOKS[@]}"; do
	path="$REPO_TOP/$HOOK_DIR/$hook"
	if [ ! -f "$path" ]; then
		printf 'ERROR: %s does not exist\n' "$path" >&2
		all_ok=1
	elif [ ! -x "$path" ]; then
		printf 'ERROR: %s is not executable\n' "$path" >&2
		all_ok=1
	fi
done

if [ "$all_ok" -eq 0 ]; then
	printf 'hooks: all ok\n'
fi

exit "$all_ok"
