# Kampliment

Kampliment is a tool to control [Kakoune](https://github.com/mawww/kakoune) editor from the command line.

## Installation

### From source

Requires [Rust](https://www.rust-lang.org) installed on your system.

Clone the repository and run `cargo install`

## Kakoune integration

Add following definition into your kakrc.

```kak
evaluate-commands %sh{
    kamp init -a -e EDITOR='kamp edit'
}
```

### Kakoune mappings example

```kak
map global normal -docstring 'terminal' <c-t> ': connect terminal<ret>'
map global normal -docstring 'files'    <c-f> ': connect terminal-popup kamp-files<ret>'
map global normal -docstring 'buffers'  <c-b> ': connect terminal-popup kamp-buffers<ret>'
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
alias kft='kamp get -b \* opt filetype | sort | uniq' # list file types you're working on
```

## Provided scripts

| script                                   | function                         |
| ---------------------------------------- | -------------------------------- |
| [`kamp-buffers`](scripts/kamp-buffers)   | pick buffers                     |
| [`kamp-files`](scripts/kamp-files)       | pick files                       |
| [`kamp-gitls`](scripts/kamp-gitls)       | pick from `git ls-files`         |
| [`kamp-sessions`](scripts/kamp-sessions) | attach session and pick a buffer |

## Similar projects

- [kks](https://github.com/kkga/kks)
- [kakoune.cr](https://github.com/alexherbo2/kakoune.cr)
- [kakoune-remote-control](https://github.com/danr/kakoune-remote-control)
