# Configuration

`bindfinder` reads runtime configuration from YAML.

Default config path:

```bash
~/.config/bindfinder/config.yaml
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

integration:
  mode: "auto"
  launch_key: "ctrl-/"
  tmux:
    enabled: true
    key: "/"
    use_popup: true
    popup_width: "80%"
    popup_height: "80%"
  shell:
    enabled: true
    preferred: "auto"
    binding: "ctrl-/"
  terminal:
    enabled: false
    preferred: "auto"
```

Supported key names:

- `q`
- `k`
- `j`
- `up`
- `down`
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
