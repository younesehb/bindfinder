# Navi Support

`bindfinder` can import and search navi-style `.cheat` repositories.

Use navi imports for command/snippet content. For reusable bindfinder YAML packs
that can include both commands and keybindings, use `bindfinder packs import`.

## Featured Repositories

The featured repository list is based on the curated list published in
`denisidoro/cheats/featured_repos.txt`.

Show the embedded featured list:

```bash
bindfinder navi featured
```

## Import

Import a repository using GitHub shorthand:

```bash
bindfinder navi import denisidoro/cheats
```

Import using HTTPS:

```bash
bindfinder navi import https://github.com/denisidoro/cheats
```

Import using SSH:

```bash
bindfinder navi import git@github.com:denisidoro/cheats
```

If the repository was already imported, `bindfinder` runs a fast-forward pull.

## Storage

Imported repositories are stored by default in:

```bash
~/.config/bindfinder/repos
```

Override with:

```bash
BINDFINDER_NAVI_REPOS_DIR=/path/to/repos
```

## Format Support

Current support targets the common navi/cheats `.cheat` format:

- `% section` headers become tags
- `# description` lines become entry titles
- following command lines become searchable commands
- trailing `$ variable` definitions are ignored for now

This is enough to make the `denisidoro/cheats` repository searchable inside the
current `bindfinder` TUI and CLI.
