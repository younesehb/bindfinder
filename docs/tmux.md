# tmux Integration

`bindfinder` works well inside `tmux` and supports both split-pane and popup
launch modes.

The default is split mode. That matches the project's terminal-first baseline:
full-screen or pane takeover everywhere, popup overlays only where explicitly
enabled.

## Recommended Setup

Write the managed tmux block automatically:

```bash
bindfinder install tmux --write
tmux source-file ~/.tmux.conf
```

In the current implementation, the installed tmux binding calls the internal
`tmux-launch` flow, which:

- asks tmux for the current pane id
- opens `bindfinder` in a new split or popup
- captures the selected command
- pastes it back into the original pane
- closes the temporary pane when appropriate

## Default Binding

The tmux config stores only the key after your prefix.

The default tmux key is:

```tmux
prefix + C-]
```

If your tmux prefix is `C-a`, that means:

```text
Ctrl-a Ctrl-]
```

## Split Mode (Default)

When `integration.tmux.use_popup: false`, the generated binding looks like:

```tmux
bind-key C-] run-shell "/home/USER/.local/bin/bindfinder tmux-launch"
```

The `tmux-launch` subcommand opens a vertical split sized for the TUI and then
reinjects the selected command into the original pane.

## Popup Mode

When `integration.tmux.use_popup: true`, `bindfinder` still uses the same
`tmux-launch`/`tmux-capture` flow, but opens through tmux popup support instead
of a split.

Use popup mode only if you prefer an overlay-like experience inside tmux.

## Debugging

If tmux integration misbehaves, enable:

```yaml
integration:
  tmux:
    debug: true
```

Logs are written to:

```bash
~/.cache/bindfinder/tmux-capture.log
```

One-off override:

```bash
BINDFINDER_DEBUG_LOG=/tmp/bindfinder.log
```

## Notes

- `bind-key` is a tmux command, not a bash command.
- The selected command is pasted back into the original tmux pane, not executed.
- If the binding changes do not take effect, reload tmux with `tmux source-file ~/.tmux.conf`.
