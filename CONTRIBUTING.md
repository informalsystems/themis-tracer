# Contributing

## Tests

See [./tests/README.md](./tests/README.md).

## Development Environment

Install [direnv](https://direnv.net/).

### `rustup`

Install rustup

```sh
curl https://sh.rustup.rs -sSf | sh
```

We are currently working with our toolchain and components pinned. These
constraints are defined in [./rust-toolchain](./rust-toolchain) and they should
be picked up by your rust development environment automatically.

### Editors

#### Doom Emacs (using lsp)

Edit your [~/.doom.d/init.el](~/.doom.d/init.el), uncommenting `rust`, and add
the `lsp` option:

```emacs-lisp
(rust +lsp)
```
