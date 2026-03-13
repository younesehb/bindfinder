# Codex Skill Usage

`bindfinder` ships with a Codex skill for creating tool entries from real local configs:

- `$bindfinder-key-packs`

## Install The Skill

If your coding agent supports skill installation, copy this:

```text
Fetch and follow instructions from https://raw.githubusercontent.com/younesehb/bindfinder/main/.codex/INSTALL.md
```

Then restart Codex so the new skill is picked up.

Use it when you want Codex to inspect a tool's config files, extract the actual keybindings or commands, and turn them into bindfinder entries.

## What It Is For

Typical examples:

- "I use Vim. Find my config, extract my custom keybindings, and add them to bindfinder."
- "I use Neovim. Turn my mappings into bindfinder keys and commands."
- "Read my tmux config and add my local bindings as bindfinder entries."

## What The Skill Does

The skill guides Codex to:

1. find the real config files for the tool
2. inspect common config locations first
3. search locally if the config path is not known
4. extract mappings, actions, commands, and modes
5. normalize them into bindfinder YAML entries
6. validate and test the result

It is meant for tools like:

- Vim
- Neovim
- tmux
- zsh
- bash
- Kitty
- WezTerm

## How To Use It

Invoke the skill directly in Codex:

```text
$bindfinder-key-packs I use Vim. Find my config, extract my custom keybindings, and add them to bindfinder entries.
```

You can also be more specific:

```text
$bindfinder-key-packs Read my ~/.vimrc and convert my useful mappings into bindfinder key entries.
```

Or ask for overrides instead of built-in packs:

```text
$bindfinder-key-packs Take my Neovim mappings and add them to my bindfinder override entries.
```

## Where The Skill Writes

Depending on the task, Codex may create or update:

- built-in packs under `assets/packs/`
- key overrides via `bindfinder config keys`
- command overrides via `bindfinder config commands`
- reusable YAML pack repos for `bindfinder packs import`

## Related Docs

- [Pack format](./packs.md)
- [Configuration](./config.md)
- [tmux integration](./tmux.md)
