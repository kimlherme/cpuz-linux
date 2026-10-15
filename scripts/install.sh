#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
prefix=${HOME}/.local
offline=
uninstall=0
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix) [ "$#" -ge 2 ] || { echo 'Missing --prefix value' >&2; exit 2; }; prefix=$2; shift 2 ;;
        --offline) offline=--offline; shift ;;
        --uninstall) uninstall=1; shift ;;
        -h|--help) echo 'Usage: scripts/install.sh [--prefix /absolute/path] [--offline] [--uninstall]'; exit 0 ;;
        *) echo "Unknown option: $1" >&2; exit 2 ;;
    esac
done
case "$prefix" in /*) ;; *) echo 'Prefix must be an absolute path' >&2; exit 2 ;; esac
if [ "$uninstall" = 1 ]; then
    rm -f -- "$prefix/bin/cpuz-linux" "$prefix/share/man/man1/cpuz-linux.1"
    rm -rf -- "$prefix/share/doc/cpuz-linux"
    echo "Removed cpuz-linux from $prefix"
    exit 0
fi
cargo_bin=${CARGO:-cargo}
if ! command -v "$cargo_bin" >/dev/null 2>&1; then
    if [ -x "$HOME/.cargo/bin/cargo" ]; then cargo_bin=$HOME/.cargo/bin/cargo; else echo 'Cargo not found. See docs/distributions.md.' >&2; exit 1; fi
fi
cd "$root"
# The optional word is deliberately either empty or --offline, never user code.
"$cargo_bin" build --release --locked ${offline:+"$offline"}
install -Dm755 target/release/cpuz-linux "$prefix/bin/cpuz-linux"
install -Dm644 packaging/cpuz-linux.1 "$prefix/share/man/man1/cpuz-linux.1"
install -Dm644 README.md "$prefix/share/doc/cpuz-linux/README.md"
install -Dm644 LICENSE "$prefix/share/doc/cpuz-linux/LICENSE"
for doc in docs/*; do [ -f "$doc" ] || continue; install -Dm644 "$doc" "$prefix/share/doc/cpuz-linux/docs/$(basename "$doc")"; done
echo "Installed: $prefix/bin/cpuz-linux"
echo "Run: $prefix/bin/cpuz-linux --tui"
