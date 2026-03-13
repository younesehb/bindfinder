# Bindfinder Pack Shape

Use this YAML structure for new tool packs and override files.

```yaml
pack:
  id: "vim-core"
  tool: "vim"
  title: "Vim Core Keys"
  version: "0.1.0"
  source: "built-in"

entries:
  - id: "split-vertical"
    type: "binding"
    title: "Split Window Vertically"
    keys: "Ctrl-w v"
    command: "vsplit"
    description: "Create a vertical split in normal mode."
    tags: ["windows", "layout", "split"]
    aliases: ["vertical split", "split right", "new split right"]

  - id: "quit-without-save"
    type: "command"
    title: "Quit Without Saving"
    command: ":q!"
    description: "Exit the current buffer without saving changes."
    tags: ["quit", "save", "command"]
    aliases: ["force quit", "discard changes", "exit without save"]
```

## Field Guidance

- `pack.id`: stable unique identifier for the pack
- `tool`: lowercase tool name shown in results
- `title`: human-readable pack title
- `source`: usually `built-in`, `override`, or another explicit source label
- `entries[].id`: stable unique id within the pack
- `type`: `binding` or `command`
- `keys`: the key sequence for `binding` entries
- `command`: the named action, ex command, or executable command users may want to copy
- `description`: one concise sentence
- `tags`: search and grouping hints
- `aliases`: phrases users will search in natural language

## Search-Oriented Naming

Prefer these title shapes:

- `Split Window Vertically`
- `Delete Current Line`
- `Find In Current Buffer`
- `Quit Without Saving`

Avoid titles that are too low-level:

- `vsplit`
- `dd`
- `q!`

Those still belong in `keys`, `command`, and `aliases`.

## Override Flow

User overrides use the same shape and usually live in:

- `~/.config/bindfinder/overrides/keys.yaml`
- `~/.config/bindfinder/overrides/commands.yaml`

To replace an existing entry, keep the same `pack.id` and `entries[].id`.
