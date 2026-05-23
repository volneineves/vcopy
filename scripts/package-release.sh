#!/bin/sh
set -eu

dist_dir="${VCOPY_DIST_DIR:-dist}"
targets="${VCOPY_TARGETS:-x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu}"

for target in $targets; do
  case "$target" in
    x86_64-unknown-linux-gnu) arch="x86_64" ;;
    aarch64-unknown-linux-gnu) arch="aarch64" ;;
    *)
      echo "Unsupported release target: $target" >&2
      echo "Supported targets: x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu" >&2
      exit 1
      ;;
  esac

  if command -v rustup >/dev/null 2>&1; then
    if ! rustup target list --installed | grep -qx "$target"; then
      echo "Missing Rust target: $target" >&2
      echo "Install it with: rustup target add $target" >&2
      exit 1
    fi
  else
    sysroot="$(rustc --print sysroot)"
    if [ ! -d "$sysroot/lib/rustlib/$target" ]; then
      echo "Missing Rust target standard library: $target" >&2
      echo "Install the target with your Rust toolchain or distribution package manager." >&2
      exit 1
    fi
  fi

  cargo build --release --locked --target "$target"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT INT TERM

  mkdir -p "$dist_dir"
  cp "target/$target/release/vcopy" "$tmp/vcopy"
  tar -czf "$dist_dir/vcopy-linux-$arch.tar.gz" -C "$tmp" vcopy
  rm -rf "$tmp"
  trap - EXIT INT TERM

  echo "Created $dist_dir/vcopy-linux-$arch.tar.gz"
done
