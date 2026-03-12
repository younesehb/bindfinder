#!/bin/sh
set -eu

REPO="${BINDFINDER_REPO:-younesehb/bindfinder}"
VERSION="${BINDFINDER_VERSION:-latest}"
INSTALL_ROOT="${BINDFINDER_INSTALL_ROOT:-$HOME/.local}"
BIN_DIR="${BINDFINDER_BIN_DIR:-$INSTALL_ROOT/bin}"
MAN_DIR="${BINDFINDER_MAN_DIR:-$INSTALL_ROOT/share/man/man1}"
SETUP=1

say() {
  printf '%s\n' "$*"
}

step() {
  printf '==> %s\n' "$*"
}

done_step() {
  printf '  -> %s\n' "$*"
}

warn() {
  printf 'warning: %s\n' "$*" >&2
}

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

shell_reload_command() {
  shell_name="$(basename_of "${SHELL:-}")"
  case "$shell_name" in
    bash) printf '%s\n' 'source ~/.bashrc' ;;
    zsh) printf '%s\n' 'source ~/.zshrc' ;;
    fish) printf '%s\n' 'source ~/.config/fish/config.fish' ;;
    *) printf '%s\n' '' ;;
  esac
}

tmux_reload_command() {
  if [ -n "${TMUX:-}" ]; then
    printf '%s\n' 'tmux source-file ~/.tmux.conf'
  else
    printf '%s\n' ''
  fi
}

default_shortcut_hint() {
  targets="$1"
  case " $targets " in
    *" tmux "*) printf '%s\n' 'prefix + Ctrl-] (default tmux shortcut)' ;;
    *" bash "*|*" zsh "*|*" fish "*) printf '%s\n' 'Ctrl-] (default shell shortcut)' ;;
    *) printf '%s\n' '' ;;
  esac
}

run_setup() {
  targets="$1"

  step "Writing config"
  "$BIN_DIR/bindfinder" config init
  done_step "config ready"

  if [ -n "$targets" ]; then
    step "Installing shell/tmux integration"
    for target in $targets; do
      "$BIN_DIR/bindfinder" install "$target" --write
      done_step "$target integration installed"
    done
  else
    warn "skipping integration setup because the shell could not be detected safely"
    warn "run 'bindfinder install auto --write' after adding $BIN_DIR to PATH"
  fi

  if command -v git >/dev/null 2>&1; then
    step "Importing default cheats"
    "$BIN_DIR/bindfinder" navi import denisidoro/cheats >/dev/null 2>&1 || true
    done_step "denisidoro/cheats imported when available"
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

say "bindfinder installer"

step "Detecting platform"
target="$(detect_target)"
done_step "$target"

step "Resolving version"
version="$(resolve_version)"
done_step "v$version"

archive="bindfinder-${version}-${target}.tar.gz"
url="https://github.com/$REPO/releases/download/v${version}/${archive}"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

step "Downloading release"
curl -fsSL "$url" -o "$tmpdir/$archive"
done_step "$archive"

step "Unpacking archive"
tar -xzf "$tmpdir/$archive" -C "$tmpdir"

archive_dir="$tmpdir/bindfinder-${version}-${target}"
if [ ! -d "$archive_dir" ]; then
  echo "bindfinder installer: archive layout was not recognized" >&2
  exit 1
fi
done_step "$archive_dir"

step "Installing files"
mkdir -p "$BIN_DIR" "$MAN_DIR"
install_file "$archive_dir/bindfinder" "$BIN_DIR/bindfinder" 0755

if [ -f "$archive_dir/man/man1/bindfinder.1" ]; then
  install_file "$archive_dir/man/man1/bindfinder.1" "$MAN_DIR/bindfinder.1" 0644
fi
done_step "binary -> $BIN_DIR/bindfinder"
if [ -f "$MAN_DIR/bindfinder.1" ]; then
  done_step "man page -> $MAN_DIR/bindfinder.1"
fi

step "Verifying install"
if ! "$BIN_DIR/bindfinder" --version >/dev/null 2>&1; then
  rm -f "$BIN_DIR/bindfinder"
  rm -f "$MAN_DIR/bindfinder.1"
  echo "bindfinder installer: installed binary failed to start on this host" >&2
  echo "Use 'cargo install --git https://github.com/$REPO' on this platform for now." >&2
  exit 1
fi
done_step "binary starts correctly"

if [ "$SETUP" -eq 1 ]; then
  setup_target="$(detect_setup_target)"
  run_setup "$setup_target"
fi

echo
say "bindfinder installed"
if path_contains_dir "$BIN_DIR"; then
  :
else
  say "Add $BIN_DIR to your PATH, then run bindfinder."
  exit 0
fi
if [ "$SETUP" -eq 1 ]; then
  say "Run bindfinder or use your installed shortcut."
else
  say "Run bindfinder config init"
  say "Run bindfinder install auto --write"
fi
shortcut_hint="$(default_shortcut_hint "${setup_target:-}")"
if [ -n "$shortcut_hint" ]; then
  say "Default shortcut: $shortcut_hint"
fi
say "More info: bindfinder --help"
say "Docs: https://github.com/$REPO/tree/main/docs"
if [ "$SETUP" -eq 1 ]; then
  reload_cmd="$(shell_reload_command)"
  tmux_cmd="$(tmux_reload_command)"
  if [ -n "$tmux_cmd" ]; then
    say "Reload tmux now: $tmux_cmd"
  fi
  if [ -n "$reload_cmd" ]; then
    say "Reload your shell now: $reload_cmd"
  else
    say "Reload your current shell session now."
  fi
fi
