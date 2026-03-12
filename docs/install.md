# Installation

`bindfinder` is terminal-only and works on Linux and macOS.

Current supported install paths:

- `curl | sh` installer from GitHub
- `cargo install`
- Homebrew from a local checkout
- prebuilt release tarballs from GitHub Releases
- local man page install with `bindfinder install man --write`

## Recommended

Install the latest release into `~/.local`:

```bash
curl -fsSL https://raw.githubusercontent.com/younesehb/bindfinder/main/install.sh | sh
```

The installer:

- detects the current platform
- downloads the matching release archive from GitHub
- installs `bindfinder` into `~/.local/bin`
- installs the man page into `~/.local/share/man/man1`
- writes the default config with `bindfinder config init`
- installs tmux, bash, zsh, or fish integration automatically when the current environment can be detected safely

Useful overrides:

```bash
BINDFINDER_VERSION=0.1.1 curl -fsSL https://raw.githubusercontent.com/younesehb/bindfinder/main/install.sh | sh
BINDFINDER_INSTALL_ROOT="$HOME/.local" curl -fsSL https://raw.githubusercontent.com/younesehb/bindfinder/main/install.sh | sh
curl -fsSL https://raw.githubusercontent.com/younesehb/bindfinder/main/install.sh | sh -s -- --no-setup
```

If your shell does not already include `~/.local/bin` on `PATH`, add it first.
After setup, reload your shell or tmux config once.
If the installer cannot determine the right shell integration target, it leaves
the binary installed and prints the follow-up command to run manually.

After changing integration-related config later, you can re-apply it with:

```bash
bindfinder reload
```

## Cargo

If the project is not yet published to crates.io, install directly from GitHub:

```bash
cargo install --git https://github.com/younesehb/bindfinder
```

If it is published later, users can switch to:

```bash
cargo install bindfinder
```

## Release Tarballs

Tagged releases ship prebuilt tarballs for:

- Linux `x86_64-unknown-linux-gnu`
- macOS `aarch64-apple-darwin`

Intel macOS is still supported through source installs with Cargo or Homebrew,
but the current GitHub release automation does not build a separate Intel macOS
artifact.

Install by unpacking the archive and placing `bindfinder` somewhere on `PATH`,
for example:

```bash
tar -xzf bindfinder-<version>-<target>.tar.gz
install -m 0755 bindfinder-<version>-<target>/bindfinder ~/.local/bin/bindfinder
```

## Homebrew

A Homebrew formula is shipped in:

```bash
Formula/bindfinder.rb
```

Because the repository is still private, the formula is currently intended for
local use from a checked-out copy of the repository:

```bash
brew install --build-from-source ./Formula/bindfinder.rb
```

That installs the binary and the shipped man page.

Once the repository is public, you can move this formula into a tap repository
or keep it in this repository and document one of these flows:

```bash
brew tap younesehb/bindfinder https://github.com/younesehb/bindfinder
brew install bindfinder
```

or:

```bash
brew install --build-from-source https://raw.githubusercontent.com/younesehb/bindfinder/main/Formula/bindfinder.rb
```

If you want fully bottled Homebrew installs later, the next step is to create a
dedicated tap repository and attach macOS release artifacts to GitHub Releases.

## Man Page

`cargo install` does not install man pages automatically. `bindfinder` ships a
man page and can install it itself:

```bash
bindfinder install man --write
man bindfinder
```

Override the destination with:

```bash
BINDFINDER_MANPAGE_DIR=/custom/man/man1 bindfinder install man --write
```

## Default Paths

Linux defaults:

- config: `~/.config/bindfinder/config.yaml`
- state: `~/.config/bindfinder/state.yaml`
- packs: `~/.config/bindfinder/packs`
- repos: `~/.local/share/bindfinder/repos`
- cache log: `~/.cache/bindfinder/tmux-capture.log`

macOS defaults:

- config: `~/Library/Application Support/bindfinder/config.yaml`
- state: `~/Library/Application Support/bindfinder/state.yaml`
- packs: `~/Library/Application Support/bindfinder/packs`
- repos: `~/Library/Application Support/bindfinder/repos`
- cache log: `~/Library/Caches/bindfinder/tmux-capture.log`

`XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_DATA_HOME`, and the existing
`BINDFINDER_*` overrides are still respected when set.

## Terminal Integrations

For shell or tmux setup:

```bash
bindfinder config init
bindfinder install auto --write
```

Then reload the relevant shell or tmux config once.
