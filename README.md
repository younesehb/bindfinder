# bindfinder

`bindfinder` is a terminal command palette for shell, `tmux`, and SSH workflows.
Open it, search for a tool like `tmux` or `git`, and insert the selected
command back into your prompt.

## Install

Recommended:

```bash
curl -fsSL https://raw.githubusercontent.com/younesehb/bindfinder/main/install.sh | sh
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

Reload your shell or tmux config once.

## Use

```bash
bindfinder
```

The TUI starts in search mode. Type immediately to filter, press `Enter` to
select, `Esc` for normal mode, and `/` to return to search.

Useful commands:

```bash
bindfinder doctor
bindfinder search tmux split
bindfinder install man --write
bindfinder navi import denisidoro/cheats
```

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
- Prebuilt release automation currently targets Linux `x86_64` and macOS Apple Silicon. Intel macOS can still install from source with Cargo or Homebrew.
- `cargo install` does not install the man page automatically. Use `bindfinder install man --write`.
- The installer script downloads release artifacts from GitHub and installs into `~/.local` by default.
- The repository ships a Homebrew formula in [Formula/bindfinder.rb](./Formula/bindfinder.rb).
