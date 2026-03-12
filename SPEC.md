# bindfinder Spec

## Product Summary

`bindfinder` is an open source terminal UI for browsing command references and
keybindings, optimized for use over SSH and inside `tmux`. The primary workflow
is opening the tool from a shell or `tmux` popup, typing a tool name such as
`tmux`, and fuzzy-searching structured entries with a preview pane.

## User Problem

Terminal users often remember that a command or binding exists but do not
remember the exact syntax or key sequence. Existing tools solve parts of this:

- `fzf` is a strong selector but not a structured knowledge system
- `tldr` is concise but not optimized for interactive browsing
- `navi` is powerful for snippets but not a universal command-reference layer

`bindfinder` should provide a single terminal-native recall layer that is fast enough
to invoke habitually.

## Requirements

### Functional

- Launch as a standalone CLI/TUI
- Work correctly over SSH
- Work inside `tmux`, including popup mode
- Search across structured entries
- Support built-in packs and local user packs
- Output selected content for copy/paste or command insertion flows

### Non-functional

- Single binary distribution
- Fast startup
- Low memory footprint
- No desktop dependency
- Terminal compatibility across common developer setups

## Architecture

### CLI Layer

Responsible for command parsing and execution mode selection.

Initial commands:

- `bindfinder`
- `bindfinder search <query>`
- `bindfinder list tools`
- `bindfinder validate <pack>`

### Core Layer

Shared domain types and pack loading:

- pack schema
- entry normalization
- source identity
- validation rules

### Search Layer

In-memory fuzzy matching and scoring across:

- tool names
- entry titles
- command strings
- aliases and tags

### TUI Layer

Terminal UI with:

- search input
- results pane
- preview pane
- footer key hints

### Integration Layer

Terminal workflow integration:

- `tmux` popup
- `tmux` split fallback
- shell helpers later

## Data Model

Each normalized entry should include:

- `id`
- `tool`
- `title`
- `type`
- `keys`
- `command`
- `description`
- `examples`
- `tags`
- `source`
- `platform`
- `aliases`

Entry types:

- `binding`
- `command`
- `snippet`
- `workflow`
- `note`

## Pack Format

Packs are YAML files that define metadata and entries. Each pack corresponds to
one tool or one cohesive topic.

The initial pack loader should support:

- pack metadata
- per-entry aliases and tags
- examples as a list of strings

## MVP Milestones

1. Compile and run a minimal Rust TUI shell
2. Finalize pack schema and add built-in `tmux` pack
3. Load built-in packs at startup
4. Add fuzzy filtering and navigation
5. Add preview pane rendering
6. Document `tmux` popup integration

## Risks

- Terminal rendering differences across environments
- Popup behavior varying by `tmux` version
- Scope drift into general shell launching
- Weak search quality if content normalization is poor

## Success Criteria

- Tool opens reliably in terminal and inside `tmux`
- Basic search is fast and useful
- `tmux` entries are immediately useful to a real user
- Pack format is simple enough for outside contributors
