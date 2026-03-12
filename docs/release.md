# Release

## Version

Current version: `0.1.3`

## Targets

The project now ships release automation for:

- `x86_64-unknown-linux-gnu`
- `aarch64-apple-darwin`

Intel macOS remains installable from source, but is not currently included in
the automated release matrix because the `macos-13` runner is not available for
this repository.

The workflow is in [.github/workflows/release.yml](../.github/workflows/release.yml)
and runs on version tags such as `v0.1.0`.

For Homebrew, the repository also ships a formula in
[Formula/bindfinder.rb](../Formula/bindfinder.rb). It can be used from a
checked-out copy of the repo, moved into a tap, or referenced directly from the
repository.

## Local Release Steps

Run tests:

```bash
. "$HOME/.cargo/env"
cargo test
```

Build the release binary:

```bash
. "$HOME/.cargo/env"
cargo build --release
```

Package a Linux release tarball locally:

```bash
mkdir -p dist/bindfinder-0.1.3-x86_64-unknown-linux-gnu
cp target/release/bindfinder dist/bindfinder-0.1.3-x86_64-unknown-linux-gnu/
cp README.md LICENSE CHANGELOG.md dist/bindfinder-0.1.3-x86_64-unknown-linux-gnu/
tar -C dist -czf dist/bindfinder-0.1.3-x86_64-unknown-linux-gnu.tar.gz bindfinder-0.1.3-x86_64-unknown-linux-gnu
```

## Publish Gap

GitHub Releases are now expected to be published by the release workflow on tag
push. Local manual packaging is still useful for smoke tests and one-off builds.
