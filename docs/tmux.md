# tmux Integration

`bindfinder` is intended to be especially effective inside `tmux`.

## Popup Mode

Preferred integration:

```tmux
bind-key / display-popup -E 'bindfinder'
```

This requires a `tmux` version with popup support.

## Split Fallback

For older environments:

```tmux
bind-key / split-window -v 'bindfinder'
```

## Design Notes

- The app must handle narrow terminals gracefully.
- Startup time matters more than visual complexity.
- Popup mode should be the primary documented workflow.
