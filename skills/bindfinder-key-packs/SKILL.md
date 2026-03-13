---
name: bindfinder-key-packs
description: Create or update bindfinder YAML packs and override files for tool keybindings and command references. Use when a user wants to add a new tool like Vim, Neovim, tmux, shell, or Git to bindfinder; when converting a user's real config files into searchable `binding` or `command` entries; or when editing `bindfinder config keys` / `bindfinder config commands` content.
---

# Bindfinder Key Packs

Create bindfinder content for tools that people search by action, command, or shortcut. This skill is for authoring new YAML packs, overrides, and reusable examples that make tool usage searchable in bindfinder, especially from a user's real local config.

## Use This Skill When

- adding a new tool pack such as Vim, Neovim, Git, Docker, or Kitty
- a user says "I use this tool, get my keybindings from my config and add them to bindfinder"
- converting a tool's docs or config into bindfinder `binding` entries
- adding matching `command` entries so searches return both commands and keys
- editing user override files opened by `bindfinder config keys` or `bindfinder config commands`
- normalizing shortcut names and search aliases so results are easy to find

## Workflow

1. Find the user's real source of truth.
   Prefer the user's actual config over generic docs. If the user does not give the path, inspect common config locations first, then search locally. Ask only if the location is ambiguous or the tool is highly unusual.

2. Extract the actual mappings.
   Read the config files, identify the real keybindings, modes, and actions, and normalize them into bindfinder entries. For Vim-like tools, include both mappings and important ex commands when they help search.

3. Choose the destination.
   - Built-in reusable pack: add a new YAML pack in the repo's `assets/packs/`
   - User override: edit `~/.config/bindfinder/overrides/keys.yaml` or `commands.yaml`
   - Reusable external repo: create a standalone YAML pack repo that `bindfinder packs import` can load

4. Normalize the entries.
   - use `type: binding` for shortcuts and keymaps
   - use `type: command` for CLI commands and arguments
   - keep `title` short and search-oriented
   - put the actual shortcut in `keys`
   - put the action or executable command in `command`
   - add `aliases` for what users will actually search

5. Pair commands and keys when both matter.
   For tools like Vim and tmux, users often need both:
   - the keybinding to trigger the action
   - the command form or named action for understanding/search

6. Validate and test.
   - run `bindfinder validate <pack.yaml>` for standalone files
   - for repo work, also run `bindfinder search --type keys <tool> <query>` and `bindfinder search --type commands <tool> <query>`

## Config Discovery

When the task is "take my tool config and add its keys," use this order:

1. Check common locations for that tool.
2. Search the home directory or repo with `rg --files` if needed.
3. Read only the files needed to identify mappings.
4. Prefer the user's custom mappings over default docs.
5. If mappings are spread across multiple files, merge them into one coherent pack or override file.

Use the config discovery reference for common locations and shell commands:

- `references/local-config-discovery.md`

## Authoring Rules

- One pack should cover one tool or one tight topic area.
- `pack.id` must stay stable.
- `entries[].id` must be unique within the pack and stable across edits.
- Prefer concise descriptions over long explanations.
- Add aliases for user intent, not just official terminology.
- Do not invent shortcuts. If uncertain, mark the source in the description or stop and verify.
- Keep key strings readable: `Ctrl-w v`, `Shift-Left`, `Leader + ff`, `gg`, `:wq`
- If a tool has multiple modes, encode that in the title or description.
- If mappings come from user config, prefer titles based on action, not the raw key sequence.
- If the same action has both a keybinding and a command form, include both when they help search.

## What Good Entries Look Like

- A user searching `vim split vertical` should find both the split command and the keybinding.
- A user searching `delete line` should match the Vim action even if they do not know `dd`.
- A user searching `quit without save` should match `:q!`.

## References

- Pack schema and naming patterns: `references/pack-shape.md`
- Common config locations and local discovery: `references/local-config-discovery.md`
- Vim-oriented example pack: `references/vim-example.md`

Load only the reference you need.
