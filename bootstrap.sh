#!/usr/bin/env bash
#
# setmeup bootstrap launcher — thin launcher that ONLY places the setmeup
# binary on a fresh machine and hands off to the first-run wizard
# (`setmeup configure`). Provisioning logic lives in the binary, never here.
#
# Ubuntu/WSL2 & macOS: ensure the Rust toolchain (if missing), build from
# source, launch the wizard. Windows: delegate to WSL2 (installing it first
# if missing). Git Bash is never a supported environment.

set -eu

REPO_URL='https://github.com/mdecurtins/setmeup.git'
RAW_URL='https://raw.githubusercontent.com/mdecurtins/setmeup/main/bootstrap.sh'
BIN_NAME='setmeup'
WIZARD_COMMAND='configure'

print_err() { printf 'bootstrap: %s\n' "$*" >&2; }
die() {
	print_err "$*"
	exit 1
}

# Script dir (empty when piped via `curl ... | sh`): lets us detect a clone.
script_dir='.'
if [ -f "$0" ]; then
	script_dir="$(cd "$(dirname "$0")" && pwd)"
fi

# WSL kernels expose an interop marker and a Microsoft-branded kernel version.
is_wsl() {
	[ -f '/proc/sys/fs/binfmt_misc/WSLInterop' ] && return 0
	case "$(uname -r)" in
	*microsoft* | *WSL*) return 0 ;;
	esac
	return 1
}

detect_os() {
	case "$(uname -s)" in
	*MSYS* | *MINGW* | *CYGWIN*) printf 'windows\n' ;;
	Darwin) printf 'macos\n' ;;
	Linux) is_wsl && printf 'wsl2\n' || printf 'linux\n' ;;
	*) printf 'unknown\n' ;;
	esac
}

# Install the Rust toolchain only when cargo is missing (idempotent re-run).
install_cargo_if_missing() {
	if command -v cargo >/dev/null 2>&1; then
		printf 'bootstrap: cargo already present; skipping rustup install\n'
		return 0
	fi
	printf 'bootstrap: cargo not found — installing Rust via the rustup installer\n'
	_tmp_dir="$(mktemp -d)"
	trap 'rm -rf "$_tmp_dir"' 0 1 2 3 15
	if ! curl --proto '=https' --tlsv1.2 -sSf 'https://sh.rustup.rs' -o "$_tmp_dir/rustup-init.sh"; then
		die 'failed to download the rustup installer — check your network connection'
	fi
	if ! sh "$_tmp_dir/rustup-init.sh" -y; then
		die 'rustup installation failed — see the output above'
	fi
	# rustup installs into ~/.cargo/bin; this run is non-interactive, so extend PATH.
	export PATH="$HOME/.cargo/bin:$PATH"
	if ! command -v cargo >/dev/null 2>&1; then
		die 'cargo still not on PATH after rustup — add ~/.cargo/bin and re-run'
	fi
}

# Install the fresh binary beside cargo: ~/.cargo/bin, /usr/local/bin, or in place.
install_binary() {
	_bin_src="$1"
	if [ -d "$HOME/.cargo/bin" ]; then
		_bin_dir="$HOME/.cargo/bin"
	elif [ -w '/usr/local/bin' ]; then
		_bin_dir='/usr/local/bin'
	else
		_bin_dir="$(dirname "$(command -v cargo)")"
	fi
	install -m 0755 "$_bin_src" "$_bin_dir/$BIN_NAME"
	export PATH="$_bin_dir:$PATH"
	printf 'bootstrap: installed %s at %s\n' "$BIN_NAME" "$_bin_dir/$BIN_NAME"
}

ensure_binary() {
	if [ -f "$script_dir/Cargo.toml" ]; then
		# Build local sources in place; avoids re-cloning via `cargo install`.
		printf 'bootstrap: local checkout detected — building from %s\n' "$script_dir"
		cargo build --release --manifest-path "$script_dir/Cargo.toml"
		install_binary "$script_dir/target/release/$BIN_NAME"
	else
		printf 'bootstrap: building %s from %s\n' "$BIN_NAME" "$REPO_URL"
		cargo install --git "$REPO_URL"
	fi
}

# Unix path for Ubuntu/WSL2/macOS: toolchain, build, wizard exec.
unix_bootstrap() {
	install_cargo_if_missing
	ensure_binary
	if ! command -v "$BIN_NAME" >/dev/null 2>&1; then
		die "install done but $BIN_NAME is not on PATH — open a new shell and run '$BIN_NAME $WIZARD_COMMAND'"
	fi
	exec "$BIN_NAME" "$WIZARD_COMMAND"
}

# WSL2 auto-mounts Windows drives at /mnt/<drive-letter>; map a path onto it.
wsl_make_path() {
	_win_path="$1"
	_drive="$(printf '%s' "$_win_path" | cut -c1 | tr '[:upper:]' '[:lower:]')"
	_rest="$(printf '%s' "$_win_path" | cut -c3-)"
	printf '/mnt/%s/%s\n' "$_drive" "$(printf '%s' "$_rest" | sed 's|\\|/|g')"
}

# `wsl --list --quiet` prints one line per distro; nothing means "not set up".
wsl_has_distro() {
	_wsl_distros="$(MSYS2_ARG_CONV_EXCL='*' wsl.exe --list --quiet 2>/dev/null || true)"
	[ -n "$_wsl_distros" ]
}

# Re-run this script inside WSL2 (MSYS2_ARG_CONV_EXCL stops Git Bash from
# rewriting the /mnt/... path arguments).
windows_run_in_wsl() {
	if [ -f "$0" ]; then
		_wsl_script="$(wsl_make_path "$(cygpath -w "$script_dir/$(basename "$0")")")"
		printf 'bootstrap: handing off to WSL2 (%s)\n' "$_wsl_script"
		if MSYS2_ARG_CONV_EXCL='*' wsl.exe bash -lc "exec /bin/sh '$_wsl_script'"; then
			return 0
		fi
		print_err "bootstrap.sh not reachable from inside WSL2 (mapped to $_wsl_script)"
	fi
	printf '%s\n' 'Open a WSL2 terminal and run the Unix bootstrap there:' '' "  curl -fsSL '$RAW_URL' | sh"
	return 0
}

# Native Windows without WSL2: start the installer, then print manual steps.
windows_install_wsl2() {
	printf '%s\n' \
		'bootstrap: WSL2 missing — setmeup requires WSL2 (Git Bash is not supported).' \
		'bootstrap: starting the official WSL2 installer now...'
	_install_rc=0
	MSYS2_ARG_CONV_EXCL='*' powershell.exe -NoProfile -Command 'wsl --install' || _install_rc=$?
	printf '%s\n' \
		'' \
		'Finish the WSL2 setup (a reboot is usually needed) and install a distro with' \
		'"wsl --install -d Ubuntu", then run the bootstrap from inside WSL2:' \
		'' \
		"  curl -fsSL '$RAW_URL' | sh" \
		'  Manual reference: https://learn.microsoft.com/windows/wsl/install'
	return "$_install_rc"
}

windows_bootstrap() {
	if command -v wsl.exe >/dev/null 2>&1 && wsl_has_distro; then
		windows_run_in_wsl
	else
		windows_install_wsl2
	fi
}

main() {
	_os="$(detect_os)"
	printf 'bootstrap: detected platform: %s\n' "$_os"
	case "$_os" in
	windows) windows_bootstrap ;;
	macos | wsl2 | linux) unix_bootstrap ;;
	*) die 'unsupported platform — setmeup supports Ubuntu/WSL2, macOS, and Windows via WSL2' ;;
	esac
}

main "$@"
