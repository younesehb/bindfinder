# Changelog

## 0.1.3 - 2026-03-12

- add `bindfinder uninstall` to remove installed files and managed integration blocks
- add `bindfinder uninstall --purge-data` to remove local config, state, packs, repos, and cache files
- document install and uninstall more clearly

## 0.1.2 - 2026-03-12

- add in-TUI argument prompts for commands with placeholders like `<branch>` or `<package>`
- make bare `bindfinder` in an interactive shell use the shell-integrated picker path
- keep CLI subcommands like `bindfinder --help`, `bindfinder doctor`, and `bindfinder reload` on the real binary path
- add `bf` as a short shell alias for the picker path
- tighten shell integration behavior and release-quality lint/test coverage

## 0.1.1 - 2026-03-12

- add favorites, hidden items, and favorites-only filtering
- make TUI start in search mode with configurable normal/search bindings
- add navi repository import support
- add tmux command reinjection and optional debug logging
- add shipped man page and `bindfinder install man --write`
- add platform-aware Linux/macOS paths, Homebrew formula, and GitHub Actions CI/release workflows
- simplify the README and move detailed usage into `docs/`

## 0.1.0 - 2026-03-12

Initial release of `bindfinder`.

- terminal-first TUI for browsing keybindings and command references
- built-in pack loading with a starter `tmux` pack
- local YAML pack loading from `~/.config/bindfinder/packs`
- YAML app config for settings, in-app keybindings, and integration preferences
- environment autodetection for `tmux`, shell, SSH, and terminal context
- `doctor`, `install auto`, and `config init` commands
- pack validation and CLI search/list commands
