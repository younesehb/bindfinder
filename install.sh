#!/bin/sh
set -eu

REPO="${BINDFINDER_REPO:-younesehb/bindfinder}"
VERSION="${BINDFINDER_VERSION:-latest}"
INSTALL_ROOT="${BINDFINDER_INSTALL_ROOT:-$HOME/.local}"
BIN_DIR="${BINDFINDER_BIN_DIR:-$INSTALL_ROOT/bin}"
MAN_DIR="${BINDFINDER_MAN_DIR:-$INSTALL_ROOT/share/man/man1}"
SETUP=1

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "bindfinder installer: missing required command: $1" >&2
    exit 1
  }
}

usage() {
  cat <<'EOF'
bindfinder installer

Usage:
  sh install.sh [--no-setup]

Options:
  --no-setup   install the binary and man page only
  --help       show this help text
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --no-setup)
      SETUP=0
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "bindfinder installer: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux) os="unknown-linux-musl" ;;
    Darwin) os="apple-darwin" ;;
    *)
      echo "bindfinder installer: unsupported operating system: $os" >&2
      exit 1
      ;;
  esac

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *)
      echo "bindfinder installer: unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  target="${arch}-${os}"

  case "$target" in
    x86_64-unknown-linux-musl|aarch64-apple-darwin)
      printf '%s\n' "$target"
      ;;
    *)
      echo "bindfinder installer: no prebuilt release for $target" >&2
      echo "Use cargo install or build from source on this platform." >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [ "$VERSION" != "latest" ]; then
    printf '%s\n' "$VERSION"
    return
  fi

  api_url="https://api.github.com/repos/$REPO/releases/latest"
  resolved="$(curl -fsSL "$api_url" | grep '"tag_name"' | head -n1 | sed -E 's/.*"([^"]+)".*/\1/')"
  if [ -z "$resolved" ]; then
    echo "bindfinder installer: failed to resolve latest release version" >&2
    exit 1
  fi
  printf '%s\n' "${resolved#v}"
}

install_file() {
  src="$1"
  dst="$2"
  mode="$3"
  if command -v install >/dev/null 2>&1; then
    install -m "$mode" "$src" "$dst"
  else
    cp "$src" "$dst"
    chmod "$mode" "$dst"
  fi
}

basename_of() {
  value="$1"
  value="${value%/}"
  value="${value##*/}"
  printf '%s\n' "$value"
}

detect_setup_target() {
  targets=""
  if [ -n "${TMUX:-}" ]; then
    targets="tmux"
  fi
  shell_name="$(basename_of "${SHELL:-}")"
  case "$shell_name" in
    bash|zsh|fish)
      if [ -n "$targets" ]; then
        targets="$targets $shell_name"
      else
        targets="$shell_name"
      fi
      ;;
    *) ;;
  esac
  printf '%s\n' "$targets"
}

run_setup() {
  targets="$1"

  "$BIN_DIR/bindfinder" config init

  if [ -n "$targets" ]; then
    for target in $targets; do
      "$BIN_DIR/bindfinder" install "$target" --write
    done
  else
    echo "bindfinder installer: skipping integration setup because the shell could not be detected safely." >&2
    echo "Run 'bindfinder install auto --write' after adding $BIN_DIR to PATH." >&2
  fi
}

path_contains_dir() {
  case ":${PATH:-}:" in
    *:"$1":*) return 0 ;;
    *) return 1 ;;
  esac
}

need_cmd curl
need_cmd tar
need_cmd uname
need_cmd mktemp

target="$(detect_target)"
version="$(resolve_version)"
archive="bindfinder-${version}-${target}.tar.gz"
url="https://github.com/$REPO/releases/download/v${version}/${archive}"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

curl -fsSL "$url" -o "$tmpdir/$archive"
tar -xzf "$tmpdir/$archive" -C "$tmpdir"

archive_dir="$tmpdir/bindfinder-${version}-${target}"
if [ ! -d "$archive_dir" ]; then
  echo "bindfinder installer: archive layout was not recognized" >&2
  exit 1
fi

mkdir -p "$BIN_DIR" "$MAN_DIR"
install_file "$archive_dir/bindfinder" "$BIN_DIR/bindfinder" 0755

if [ -f "$archive_dir/man/man1/bindfinder.1" ]; then
  install_file "$archive_dir/man/man1/bindfinder.1" "$MAN_DIR/bindfinder.1" 0644
fi

if ! "$BIN_DIR/bindfinder" --version >/dev/null 2>&1; then
  rm -f "$BIN_DIR/bindfinder"
  rm -f "$MAN_DIR/bindfinder.1"
  echo "bindfinder installer: installed binary failed to start on this host" >&2
  echo "Use 'cargo install --git https://github.com/$REPO' on this platform for now." >&2
  exit 1
fi

if [ "$SETUP" -eq 1 ]; then
  setup_target="$(detect_setup_target)"
  run_setup "$setup_target"
fi

echo
echo "bindfinder installed"
if path_contains_dir "$BIN_DIR"; then
  :
else
  echo "Add $BIN_DIR to your PATH, then run bindfinder."
  exit 0
fi
if [ "$SETUP" -eq 1 ]; then
  echo "Run bindfinder or use your installed shortcut."
else
  echo "Run bindfinder config init"
  echo "Run bindfinder install auto --write"
fi
echo "More info: bindfinder --help"
echo "Docs: https://github.com/$REPO/tree/main/docs"
if [ "$SETUP" -eq 1 ]; then
  echo "If the current shell session does not pick up the integration yet, reload it once."
fi
