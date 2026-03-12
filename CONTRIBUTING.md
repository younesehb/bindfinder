# Contributing

Thanks for contributing to `bindfinder`.

## Before Opening a PR

- run `cargo test`
- keep changes focused
- update docs if behavior or config changed
- add or update tests when fixing a bug or changing key behavior

## Development

Run the app:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

Install the local binary:

```bash
cargo install --path . --root "$HOME/.local" --force
```

## Project Areas

- `src/tui/`: terminal UI behavior
- `src/config/`: config parsing and defaults
- `src/integration/`: shell, tmux, install helpers
- `src/core/`: catalog, pack loading, navi import
- `docs/`: user-facing documentation

## Pull Requests

- describe the user-visible change clearly
- mention any config or integration impact
- include manual verification steps for shell/tmux behavior when relevant

## Releases

Release and packaging notes live in [docs/release.md](./docs/release.md).
