# Vim Example

This is a practical pattern for a Vim pack that mixes normal-mode bindings with ex commands.

```yaml
pack:
  id: "vim-core"
  tool: "vim"
  title: "Vim Core Keys"
  version: "0.1.0"
  source: "built-in"

entries:
  - id: "delete-line"
    type: "binding"
    title: "Delete Current Line"
    keys: "dd"
    command: "delete line"
    description: "Delete the current line in normal mode."
    tags: ["editing", "delete", "line"]
    aliases: ["remove line", "cut line"]

  - id: "yank-line"
    type: "binding"
    title: "Yank Current Line"
    keys: "yy"
    command: "yank line"
    description: "Copy the current line in normal mode."
    tags: ["copy", "yank", "line"]
    aliases: ["copy line"]

  - id: "paste-after"
    type: "binding"
    title: "Paste After Cursor"
    keys: "p"
    command: "paste"
    description: "Paste after the cursor in normal mode."
    tags: ["paste", "editing"]
    aliases: ["put", "paste below"]

  - id: "search-forward"
    type: "binding"
    title: "Search Forward"
    keys: "/"
    command: "search forward"
    description: "Start a forward search from normal mode."
    tags: ["search", "navigation"]
    aliases: ["find text", "search in file"]

  - id: "split-vertical"
    type: "binding"
    title: "Split Window Vertically"
    keys: "Ctrl-w v"
    command: "vsplit"
    description: "Create a vertical split in normal mode."
    tags: ["windows", "layout", "split"]
    aliases: ["vertical split", "split right"]

  - id: "write-and-quit"
    type: "command"
    title: "Write And Quit"
    command: ":wq"
    description: "Save the current file and quit."
    tags: ["save", "quit", "command"]
    aliases: ["save and exit", "write quit"]
```

## Good Vim Coverage Areas

Start with the actions people forget:

- navigation: start/end of line, word jumps, search
- editing: delete, copy, paste, undo, redo
- windows: split, move between splits, resize
- files: write, quit, write and quit, quit without saving
- visual mode: select line/block, indent, shift

## Practical Heuristics

- Use the action name as the `title`, not the raw key.
- Put the raw Vim key sequence in `keys`.
- Put the ex command or normalized action in `command` when useful.
- Mention the mode in the description if it is not obvious.
- Add aliases for intent, e.g. `quit without save`, not just `:q!`.
