# Local Config Discovery

Use this reference when the user wants bindfinder entries generated from their actual tool config.

## Discovery Order

1. Check the tool's common config locations.
2. If not found, search with `rg --files`.
3. Read only the relevant files.
4. Extract mappings, commands, modes, and comments that explain intent.

Prefer the user's local config over generic defaults.

## Common Locations

### Vim

- `~/.vimrc`
- `~/.vim/vimrc`
- `~/.config/vim/vimrc`
- `~/.vim/plugin/**/*.vim`
- `~/.vim/after/plugin/**/*.vim`

### Neovim

- `~/.config/nvim/init.vim`
- `~/.config/nvim/init.lua`
- `~/.config/nvim/lua/**/*.lua`
- `~/.config/nvim/plugin/**/*.lua`

### tmux

- `~/.tmux.conf`
- `~/.config/tmux/tmux.conf`

### zsh

- `~/.zshrc`
- `~/.zprofile`
- `~/.zsh/**/*.zsh`

### bash

- `~/.bashrc`
- `~/.bash_profile`
- `~/.bash_aliases`

### Kitty

- `~/.config/kitty/kitty.conf`

### WezTerm

- `~/.wezterm.lua`
- `~/.config/wezterm/wezterm.lua`

## Useful Search Commands

Find likely config files:

```bash
rg --files ~/.config ~ | rg 'vim|nvim|tmux|kitty|wezterm|zsh|bash'
```

Search for Vim mappings:

```bash
rg -n '(^|\\s)(noremap|nnoremap|vnoremap|inoremap|map)\\b' ~/.vimrc ~/.vim ~/.config/vim ~/.config/nvim
```

Search for tmux bindings:

```bash
rg -n '^\\s*(bind|bind-key)\\b|^\\s*set(?:-option)?\\s+-g\\s+prefix\\b' ~/.tmux.conf ~/.config/tmux
```

Search for Kitty mappings:

```bash
rg -n '^\\s*map\\s+' ~/.config/kitty/kitty.conf
```

## Extraction Guidance

- Keep the user-facing action as the title.
- Put the literal shortcut in `keys`.
- Use the mapped command or normalized action in `command`.
- Mention the mode in the description when relevant, especially for Vim or Neovim.
- Preserve custom leader keys or prefixes in the rendered shortcut text.

## Example User Request

"I use Vim. Find my config, extract my custom mappings, and add them to bindfinder."

Expected approach:

1. inspect common Vim/Neovim config locations
2. parse the mapping lines
3. normalize them into `binding` entries
4. add useful command entries like `:w`, `:q`, `:wq`, `:q!` if needed
5. validate and test with `bindfinder search --type keys vim <query>`
