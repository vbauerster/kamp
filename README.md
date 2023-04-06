# Kamp

Kamp is a tool to control [Kakoune](https://github.com/mawww/kakoune) editor from the command line.

## Installation

### From source

Requires [Rust](https://www.rust-lang.org) installed on your system.

Clone the repository and run `cargo install --path .`

## Kakoune integration

Add following definition into your kakrc.

```kak
evaluate-commands %sh{
    kamp init -a -e EDITOR='kamp edit'
}
```

## Provided scripts

The [scripts](scripts) need to be added to `$PATH` in order to use them.

| script                                     | function                         |
| ------------------------------------------ | -------------------------------- |
| [`kamp-buffers`](scripts/kamp-buffers)     | pick buffers (fzf)               |
| [`kamp-files`](scripts/kamp-files)         | pick files (fzf)                 |
| [`kamp-nnn`](scripts/kamp-nnn)             | pick files (nnn)                 |
| [`kamp-filetypes`](scripts/kamp-filetypes) | set filetype (fzf)               |
| [`kamp-lines`](scripts/kamp-lines)         | search lines in buffer (fzf)     |
| [`kamp-sessions`](scripts/kamp-sessions)   | attach session and pick a buffer |
| [`kamp-grep`](scripts/kamp-grep)           | grep interactively with fzf      |
| [`kamp-fifo`](scripts/kamp-fifo)           | pipe stdin into fifo buffer      |

### Kakoune mappings example

Following mappings use [tmux-terminal-popup](https://github.com/alexherbo2/tmux.kak/blob/716d8a49be26b6c2332ad4f3c5342e485e02dff4/docs/manual.md#tmux-terminal-popup) as popup implementation.

```kak
alias global popup tmux-terminal-popup
map global normal -docstring 'files'          <c-f> ':connect popup kamp-files<ret>'
map global normal -docstring 'git ls-files'   <c-l> ':connect popup kamp-files backend=git<ret>'
map global normal -docstring 'buffers'        <c-b> ':connect popup kamp-buffers<ret>'
map global normal -docstring 'grep selection' <c-g> ':connect popup kamp-grep "query=%val{selection}"<ret>'
map global normal -docstring 'grep limit by filetype' <c-y> ':connect popup kamp-grep -t %opt{filetype}<ret>'
```

## Shell integration

You may want to set the `EDITOR` variable to `kamp edit` so that connected programs work as intended:

```sh
export EDITOR='kamp edit'
```

Some useful aliases:

```sh
alias k='kamp edit'
alias kval='kamp get val'
alias kopt='kamp get opt'
alias kreg='kamp get reg'
alias kcd-pwd='cd "$(kamp get sh pwd)"'
alias kcd-buf='cd "$(dirname $(kamp get val buffile))"'
alias kft='kamp get opt -b \* -s filetype | sort | uniq' # list file types you're working on
```

## Similar projects

- [kks](https://github.com/kkga/kks)
- [kakoune.cr](https://github.com/alexherbo2/kakoune.cr)
- [kakoune-remote-control](https://github.com/danr/kakoune-remote-control)
