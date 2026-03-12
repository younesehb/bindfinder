# Changelog

## 0.1.9 - 2026-03-13

- fix the GitHub release workflow so both Linux and macOS assets upload reliably from one final release job
- avoid parallel tag jobs racing while creating and finalizing the same GitHub release

## 0.1.8 - 2026-03-13

- make the installer import `denisidoro/cheats` by default when `git` is available
- document that the default install comes with navi cheat content

## 0.1.7 - 2026-03-13

- make the installer print the default shortcut for the detected shell or tmux setup
- make the installer print the exact shell reload command when the current session needs it
- keep post-install guidance short while still pointing users to `bindfinder --help` and the docs

## 0.1.6 - 2026-03-13

- simplify installer output so it shows only the result and the way to start using `bindfinder`
- point users to `bindfinder --help` and the GitHub docs URL after install

## 0.1.5 - 2026-03-13

- publish `install.sh` as a release asset
- switch README and install docs to the stable `releases/latest/download/install.sh` URL
- avoid installer drift between `main` and the latest tagged release

## 0.1.4 - 2026-03-13

- switch Linux release artifacts to `x86_64-unknown-linux-musl` for better portability across hosts with older glibc
- make the installer validate the installed binary before running first-time setup
- document Linux release artifacts as musl-based

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
