#! /usr/bin/env bash
#
# install-hooks.sh — idempotent git-hooks installer
#
# Sets core.hooksPath to git-hooks/ (repo-local config, per-clone) and
# ensures all three hooks are present and executable.
#
# Usage:
#   scripts/install-hooks.sh          # install
#   scripts/install-hooks.sh --verify # check only

set -eu

HOOK_DIR='git-hooks'
HOOKS=(commit-msg pre-commit pre-push)
REPO_TOP="$(cd "$(dirname "$0")/.." && pwd)"

print_status() { printf 'install-hooks: %s\n' "$*"; }

verify_hooks() {
	local missing=0
	for hook in "${HOOKS[@]}"; do
		if [ -x "$REPO_TOP/$HOOK_DIR/$hook" ]; then
			print_status "  ok   $HOOK_DIR/$hook"
		else
			print_status "  MISS $HOOK_DIR/$hook"
			missing=1
		fi
	done
	return "$missing"
}

if [ "${1:-}" = '--verify' ]; then
	print_status 'verify mode — checking hook state'
	verify_hooks
	rc=$?
	current_hooks="$(git config --local core.hooksPath 2>/dev/null || true)"
	print_status "core.hooksPath = ${current_hooks:-<not set>}"
	exit "$rc"
fi

# Ensure hooks are executable.
for hook in "${HOOKS[@]}"; do
	if [ ! -f "$REPO_TOP/$HOOK_DIR/$hook" ]; then
		print_status "MISSING $HOOK_DIR/$hook — aborting"
		exit 1
	fi
	chmod +x "$REPO_TOP/$HOOK_DIR/$hook"
done

git config core.hooksPath "$REPO_TOP/$HOOK_DIR"
print_status "core.hooksPath set to '$REPO_TOP/$HOOK_DIR'"

verify_hooks
rc=$?

# Check gitleaks availability — warn, don't fail.
if command -v gitleaks >/dev/null 2>&1; then
	print_status 'gitleaks: found'
else
	print_status 'gitleaks: NOT found — install with: cargo install gitleaks-cli'
	print_status '  or: brew install gitleaks'
	print_status 'The pre-commit hook will fail-closed until gitleaks is installed.'
fi

exit "$rc"
