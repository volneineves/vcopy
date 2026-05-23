#!/bin/sh
set -eu

repo="${VCOPY_REPO:-OWNER/vcopy}"
version="${VCOPY_VERSION:-latest}"
prefix="${VCOPY_PREFIX:-$HOME/.local}"
bin_dir="$prefix/bin"

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "$arch" in
  x86_64|amd64) arch="x86_64" ;;
  aarch64|arm64) arch="aarch64" ;;
  *)
    echo "Unsupported architecture: $arch" >&2
    exit 1
    ;;
esac

asset="vcopy-${os}-${arch}.tar.gz"
base_url="https://github.com/${repo}/releases"

if [ "$version" = "latest" ]; then
  url="${base_url}/latest/download/${asset}"
else
  url="${base_url}/download/${version}/${asset}"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$bin_dir"
curl -fsSL "$url" | tar -xz -C "$tmp"
install -m 0755 "$tmp/vcopy" "$bin_dir/vcopy"

echo "vcopy installed at $bin_dir/vcopy"
echo "Run: vcopy --version"
