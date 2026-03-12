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

## Near-Term Extensions

- `platform`
- `source_url`
- richer examples
- optional notes and related entries
