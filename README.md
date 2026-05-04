# goto

Fuzzy directory jumper.

## Usage

```
goto j <target> [root]     Jump to a directory
goto a <name> <path>       Add a namespace
goto s [-d <n>] [-r]       Show or update settings
goto l                     List namespaces and settings
```

`root` can be a namespace or a filesystem path. If omitted, searches your home directory.

## Shell Integration

The program prints the target directory to stdout. Wrap it in a shell function to actually change directories.

### Bash

```bash
goto() {
    local dest
    dest=$(command goto j "$@") || return
    cd "$dest"
}
```

### Zsh

```zsh
goto() {
    local dest
    dest=$(command goto j "$@") || return
    cd "$dest"
}
```

### Fish

```fish
function goto
    set -l dest (command goto j $argv)
    or return
    cd $dest
end
```

## Building

```bash
cargo build --release
# or
nix build .
```
