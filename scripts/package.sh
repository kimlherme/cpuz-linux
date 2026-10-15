#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
format=${1:-}
case "$format" in deb|rpm) ;; *) echo 'Usage: scripts/package.sh deb|rpm [--offline]' >&2; exit 2 ;; esac
shift
offline=
if [ "${1:-}" = --offline ]; then offline=--offline; shift; fi
[ "$#" -eq 0 ] || { echo 'Unexpected arguments' >&2; exit 2; }
tool=dpkg-deb
[ "$format" = deb ] || tool=rpmbuild
command -v "$tool" >/dev/null 2>&1 || { echo "$tool is required for $format packaging" >&2; exit 1; }
cd "$root"
cargo_bin=${CARGO:-cargo}
if ! command -v "$cargo_bin" >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then cargo_bin=$HOME/.cargo/bin/cargo; fi
"$cargo_bin" build --release --locked ${offline:+"$offline"}
stage=$(mktemp -d)
trap 'rm -rf -- "$stage"' EXIT HUP INT TERM
mkdir -p dist
if [ "$format" = deb ]; then
    case $(uname -m) in x86_64) arch=amd64 ;; aarch64) arch=arm64 ;; *) echo 'Unsupported Debian architecture' >&2; exit 1 ;; esac
    install -Dm755 target/release/cpuz-linux "$stage/deb/usr/bin/cpuz-linux"
    install -Dm644 packaging/cpuz-linux.1 "$stage/deb/usr/share/man/man1/cpuz-linux.1"
    install -Dm644 LICENSE "$stage/deb/usr/share/doc/cpuz-linux/copyright"
    install -Dm644 README.md "$stage/deb/usr/share/doc/cpuz-linux/README.md"
    for doc in docs/*; do [ -f "$doc" ] || continue; install -Dm644 "$doc" "$stage/deb/usr/share/doc/cpuz-linux/docs/$(basename "$doc")"; done
    mkdir -p "$stage/deb/DEBIAN"
    cat > "$stage/deb/DEBIAN/control" <<EOF
Package: cpuz-linux
Version: 0.2.0
Section: utils
Priority: optional
Architecture: $arch
Maintainer: cpuz-linux contributors <noreply@github.com>
Depends: libc6, libgcc-s1
Homepage: https://github.com/kimlherme/cpuz-linux
Description: Experimental procedural CPU benchmark with terminal UI
 Single and multi thread noise workload based on static investigation of
 the CPU-Z 17.01 algorithm. Scores are experimental and not official CPU-Z.
EOF
    dpkg-deb --root-owner-group --build "$stage/deb" "dist/cpuz-linux_0.2.0_${arch}.deb"
else
    mkdir -p "$stage/rpm/SOURCES" "$stage/rpm/BUILD" "$stage/rpm/BUILDROOT" "$stage/rpm/RPMS" "$stage/rpm/SRPMS" "$stage/rpm/SPECS"
    install -m755 target/release/cpuz-linux "$stage/rpm/SOURCES/cpuz-linux"
    cp README.md LICENSE packaging/cpuz-linux.1 "$stage/rpm/SOURCES/"
    tar -czf "$stage/rpm/SOURCES/docs.tar.gz" docs
    rpmbuild --define "_topdir $stage/rpm" -bb packaging/cpuz-linux.spec
    find "$stage/rpm/RPMS" -name '*.rpm' -exec cp {} "$root/dist/" \;
fi
