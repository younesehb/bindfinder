# bindfinder

`bindfinder` is a terminal-first command reference browser for SSH-heavy workflows.
It is designed to open quickly inside a shell or `tmux`, search structured
cheat-sheet content, and help users recall commands and keybindings without
leaving the terminal.

## Goals

- Fast TUI that works over SSH
- Clean `tmux` popup workflow
- Keyboard-first navigation
- Search across commands, bindings, snippets, and workflows
- Local-first data packs with an extensible import model

## Non-goals

- Desktop overlays or OS-global hotkeys
- Cloud-only data storage
- Arbitrary plugin code execution in the MVP
- General application launching

## MVP

The initial milestone focuses on:

- a runnable Rust binary
- an interactive TUI shell
- built-in and local pack loading
- ranked multi-term query filtering
- a shared pack schema
- built-in content packs, starting with `tmux`
- documentation for `tmux` integration and future pack loading

## Usage

Current bootstrap behavior:

```bash
cargo run
```

This starts the current TUI shell with built-in `tmux` content. Type to filter,
use the arrow keys to move, and exit with `q`, `Esc`, or `Ctrl-C`.

You can also use:

```bash
cargo run -- search tmux
cargo run -- list tools
cargo run -- list config
cargo run -- list sources
cargo run -- validate assets/packs/tmux.yaml
cargo run -- config init
cargo run -- doctor
cargo run -- install auto
```

Local packs are loaded from `BINDFINDER_PACK_DIR` if set, otherwise from:

```bash
~/.config/bindfinder/packs
```

Supported local pack file extensions:

- `.yaml`
- `.yml`

## Test

Run the automated tests:

```bash
cargo test
```

Run the app interactively:

```bash
cargo run
```

Run the built binary directly after compiling:

```bash
cargo build
./target/debug/bindfinder
```

Build an optimized release binary:

```bash
cargo build --release
./target/release/bindfinder
```

## Config

Runtime settings and keybindings are configured with YAML.

Default config file:

```bash
~/.config/bindfinder/config.yaml
```

Or override it:

```bash
BINDFINDER_CONFIG=/path/to/config.yaml cargo run
```

Example config:

```yaml
settings:
  result_list_width_percent: 45
  show_footer: true
  wrap_preview: true

keybindings:
  quit: ["q", "esc", "ctrl-c"]
  clear_query: ["ctrl-u"]
  move_up: ["up", "k"]
  move_down: ["down", "j"]

integration:
  mode: "auto"
  launch_key: "ctrl-/"
  tmux:
    enabled: true
    key: "/"
    use_popup: true
    popup_width: "80%"
    popup_height: "80%"
  shell:
    enabled: true
    preferred: "auto"
    binding: "ctrl-/"
  terminal:
    enabled: false
    preferred: "auto"
```

See [docs/config.md](./docs/config.md) and [examples/config.yaml](./examples/config.yaml).

## Autodetection

`bindfinder` can autodetect the current terminal environment and choose the best
integration target.

Diagnostics:

```bash
cargo run -- doctor
```

Print the recommended install snippet for the current environment:

```bash
cargo run -- install auto
```

Write a default config file:

```bash
cargo run -- config init
```

## Release

Version `0.1.0` is ready for local packaging. See [docs/release.md](./docs/release.md)
for the release build and tarball steps.

## Roadmap

1. Load built-in packs from `assets/packs/`
2. Improve ranking into full fuzzy matching
3. Add list and preview panes
4. Add `tmux` popup integration helpers
5. Add richer local/user pack workflows
6. Add `navi` importer

## Project Layout

- `src/cli/` command-line entrypoint and argument parsing
- `src/tui/` terminal UI shell
- `src/core/` domain types
- `assets/packs/` built-in reference packs
- `docs/` architecture and integration notes

See [SPEC.md](./SPEC.md) for the product and architecture plan.
