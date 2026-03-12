# bindfinder

`bindfinder` helps you find commands and keybindings from the terminal.

Open it, type a tool name like `git`, `tmux`, or `docker`, pick what you want,
and insert it back into your prompt.

If the command has placeholders like `<branch>` or `<package>`, `bindfinder`
asks for them in the TUI before inserting the final command.

## Install

Recommended:

```bash
curl -fsSL https://raw.githubusercontent.com/younesehb/bindfinder/main/install.sh | sh
```

That installs the binary, installs the man page, writes the default config, and
sets up the shell integration automatically when it can detect your environment.

Then reload your shell once:

```bash
source ~/.bashrc
```

From source:

```bash
cargo install --path .
```

Then initialize config and install the recommended integration:

```bash
bindfinder config init
bindfinder install auto --write
```

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

Useful commands:

```bash
bindfinder doctor
bindfinder reload
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
- The default shell binding is `Alt-/`.
- The default tmux binding is `prefix + /`.
- Prebuilt release automation currently targets Linux `x86_64` and macOS Apple Silicon. Intel macOS can still install from source with Cargo or Homebrew.
- `cargo install` does not install the man page automatically. Use `bindfinder install man --write`.
- The installer script downloads release artifacts from GitHub, installs into `~/.local` by default, and runs first-time setup unless `--no-setup` is used.
- To remove bindfinder again, use `bindfinder uninstall`. Add `--purge-data` to also remove config, state, packs, repos, and cache files.
- The repository ships a Homebrew formula in [Formula/bindfinder.rb](./Formula/bindfinder.rb).
