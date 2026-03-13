# Pack Format

Packs are YAML files that describe a single tool or topic area.

## Example Shape

```yaml
pack:
  id: "tmux-core"
  tool: "tmux"
  title: "tmux Core Bindings"
  version: "0.1.0"
  source: "built-in"

entries:
  - id: "split-horizontal"
    type: "binding"
    title: "Split Pane Horizontally"
    keys: 'Prefix + "'
    command: "split-window"
    description: "Split the current pane into top and bottom panes."
    examples:
      - 'Default prefix is Ctrl-b, then "'
    tags: ["panes", "layout"]
    aliases: ["split pane horizontal"]
```

## Rules

- `pack.id` should be globally unique within loaded sources
- `entries[].id` should be unique within a pack
- `type` should be one of the supported normalized entry types
- packs should prefer concise, search-oriented descriptions

## Overrides

User overrides live in:

```bash
~/.config/bindfinder/overrides
```

Override packs use the same YAML shape. To replace a built-in or imported entry:

- use the same `pack.id`
- use the same `entries[].id`

Example:

```yaml
pack:
  id: "tmux-core"
  tool: "tmux"
  title: "tmux Core Bindings"
  version: "0.1.0"
  source: "override"

entries:
  - id: "split-vertical"
    type: "binding"
    title: "Split Pane Vertically"
    keys: "Ctrl-a + |"
    command: "split-window -h"
    description: "Your preferred split binding."
    tags: ["panes", "layout", "custom"]
    aliases: ["split pane vertical", "new pane right"]
```

If an override entry id does not already exist in that pack, it is added.

## Repository Imports

`bindfinder` can also import Git repositories that contain YAML packs in this
format.

Import a repo:

```bash
bindfinder packs import owner/repo
```

List imported repos:

```bash
bindfinder packs list
```

Imported pack repositories are stored in:

```bash
~/.local/share/bindfinder/pack-repos
```

Override that location with:

```bash
BINDFINDER_PACK_REPOS_DIR=/path/to/pack-repos
```

This is the preferred import path for reusable commands and keybindings. Unlike
`navi`, these repos can carry both `command` and `binding` entries natively.

## Near-Term Extensions

- `platform`
- `source_url`
- richer examples
- optional notes and related entries
