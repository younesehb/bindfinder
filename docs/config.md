# Configuration

`bindfinder` reads runtime configuration from YAML.

Default config path:

```bash
Linux: ~/.config/bindfinder/config.yaml
macOS: ~/Library/Application Support/bindfinder/config.yaml
```

Override with:

```bash
BINDFINDER_CONFIG=/path/to/config.yaml
```

Example:

```yaml
settings:
  result_list_width_percent: 50
  show_footer: true
  wrap_preview: true

keybindings:
  quit: ["q", "esc", "ctrl-c"]
  clear_query: ["ctrl-u"]
  move_up: ["up", "k"]
  move_down: ["down", "j"]
  page_up: ["pageup", "ctrl-u"]
  page_down: ["pagedown", "ctrl-d"]
  goto_top: ["g g"]
  goto_bottom: ["shift-g"]
  select: ["enter"]
  search_mode: ["/"]
  favorite_entry: ["f"]
  hide_entry: ["x"]
  favorite_tool: ["shift-f"]
  hide_tool: ["shift-x"]
  toggle_hidden: ["z"]
  toggle_favorites_only: ["m"]

integration:
  mode: "auto"
  launch_key: "ctrl-g ctrl-b"
  tmux:
    enabled: true
    key: "/"
    use_popup: true
    popup_width: "80%"
    popup_height: "80%"
    debug: false
  shell:
    enabled: true
    preferred: "auto"
    binding: "ctrl-g ctrl-b"
  terminal:
    enabled: false
    preferred: "auto"
```

TUI behavior:

- The app starts in search mode so typing immediately updates the filter.
- In search mode, typing updates the filter, `Up`/`Down` move the selection, and `Esc` enters normal mode.
- Normal mode uses vim-style navigation: `j`/`k`, `Ctrl-d`/`Ctrl-u`, `gg`, `G`, `/`.
- Normal mode also supports user state actions: `z` toggle hidden visibility, `m` toggle favorites-only view, `f` favorite entry, `x` hide entry, `F` favorite tool, `X` hide tool.
- Press `/` to return to search mode and clear the current query.
- Most single-key TUI actions are remappable in `keybindings`.
- Multi-stroke actions are supported in YAML, for example `goto_top: ["g g"]`.

Supported key names:

- `q`
- `k`
- `j`
- `up`
- `down`
- `pageup`
- `pagedown`
- `home`
- `end`
- `left`
- `right`
- `enter`
- `tab`
- `backspace`
- `esc`

Supported modifiers:

- `ctrl`
- `alt`
- `shift`

Integration modes:

- `auto`
- `tmux`
- `shell`
- `terminal`

When you use shell integration snippets produced by `bindfinder install auto`,
the selected command is inserted into the current prompt buffer for supported
shells instead of only being printed to stdout.

When you use tmux integration, `bindfinder` uses the internal `tmux-launch` and
`tmux-capture` flow to open the picker and paste the selected command back into
the original pane.

The project also ships a man page. Cargo does not install it automatically, but
you can place it in the standard local man directory with:

```bash
bindfinder install man --write
```

For tmux debugging, you can enable:

- `integration.tmux.debug: true`

That writes tmux capture logs to:

```bash
Linux: ~/.cache/bindfinder/tmux-capture.log
macOS: ~/Library/Caches/bindfinder/tmux-capture.log
```

For one-off debugging without editing config, set:

```bash
BINDFINDER_DEBUG_LOG=/tmp/bindfinder.log
```

User state is stored in:

```bash
Linux: ~/.config/bindfinder/state.yaml
macOS: ~/Library/Application Support/bindfinder/state.yaml
```

Override it with:

```bash
BINDFINDER_STATE=/path/to/state.yaml
```

Example remap:

```yaml
keybindings:
  hide_entry: ["shift-o"]
  toggle_hidden: ["h"]
  goto_top: ["g g", "home"]
```
