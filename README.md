# bindfinder

`bindfinder` helps you find commands and keybindings from the terminal.

Open it, type a tool name like `git`, `tmux`, or `docker`, pick what you want,
and insert it back into your prompt.

If the command has placeholders like `<branch>` or `<package>`, `bindfinder`
asks for them in the TUI before inserting the final command.

## Install

Recommended:

```bash
curl -fsSL https://github.com/younesehb/bindfinder/releases/latest/download/install.sh | sh
```

That installs the binary, installs the man page, writes the default config, and
sets up the shell integration automatically when it can detect your environment.
It also imports `denisidoro/cheats` by default when `git` is available.

From source:

```bash
cargo install --path .
```

Then initialize config and install the recommended integration:

```bash
bindfinder config
bindfinder install auto --write
```

`bindfinder config` validates and reapplies the current integration automatically
when you exit the editor.

## Use

```bash
bindfinder
```

Type immediately to filter. Press `Enter` to insert the selected command.

Normal usage:

- `bindfinder` opens the picker
- type to search
- `Enter` inserts the selected command
- `Esc` switches to normal mode
- `/` returns to search mode
- placeholder commands open a small argument form before insertion
- use your installed shortcut or just run `bindfinder`

Useful commands:

```bash
bindfinder doctor
bindfinder reload
bindfinder update
bindfinder update --check
bindfinder config
bindfinder config validate
bindfinder uninstall
bindfinder uninstall --purge-data
bindfinder install all --write
bindfinder search tmux split
bindfinder install man --write
bindfinder navi import denisidoro/cheats
```

Shell helpers:

- typing `bindfinder` with no arguments in an interactive shell now uses the shell-integrated picker path
- `bindfinder doctor` and other subcommands still go to the real binary
- `bindfinder update` installs the latest released version
- `bf` is kept as a short alias for the same shell-integrated path
- the shell keybinding gives the best live in-prompt insertion flow

## Docs

- [Installation](./docs/install.md)
- [Configuration](./docs/config.md)
- [tmux integration](./docs/tmux.md)
- [navi support](./docs/navi.md)
- [Pack format](./docs/packs.md)
- [Release process](./docs/release.md)
- [Contributing](./CONTRIBUTING.md)

## Notes

- Linux and macOS are supported.
- The default experience is full-screen in the current terminal.
- tmux and terminal-specific overlays are optional enhancements.
- There are two launch keys in practice:
  - outside tmux: `Ctrl-]`
  - inside tmux: `prefix + Ctrl-]`
- `integration.shell.binding` is the shell key.
- `integration.tmux.key` is written in the same `ctrl-...]` style in YAML and translated to tmux syntax internally.
- Prebuilt release automation currently targets Linux `x86_64` and macOS Apple Silicon. Intel macOS can still install from source with Cargo or Homebrew.
- `cargo install` does not install the man page automatically. Use `bindfinder install man --write`.
- The installer script downloads release artifacts from GitHub, installs into `~/.local` by default, and runs first-time setup unless `--no-setup` is used.
- If the current shell session does not pick up the integration immediately after install, reload that shell once.
- To remove bindfinder again, use `bindfinder uninstall`. Add `--purge-data` to also remove config, state, packs, repos, and cache files.
- The repository ships a Homebrew formula in [Formula/bindfinder.rb](./Formula/bindfinder.rb).
