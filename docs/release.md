# Release

## Version

Current version: `0.1.0`

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

Package a release tarball:

```bash
mkdir -p dist/bindfinder-0.1.0-x86_64-unknown-linux-gnu
cp target/release/bindfinder dist/bindfinder-0.1.0-x86_64-unknown-linux-gnu/
cp README.md LICENSE CHANGELOG.md dist/bindfinder-0.1.0-x86_64-unknown-linux-gnu/
tar -C dist -czf dist/bindfinder-0.1.0-x86_64-unknown-linux-gnu.tar.gz bindfinder-0.1.0-x86_64-unknown-linux-gnu
```

## Publish Gap

This workspace is not currently a git repository and is not connected to a
hosting remote, so creating a public GitHub/GitLab release is still a manual
step outside this workspace.
