# Contributing

## Tests

See [./tests/README.md](./tests/README.md).

## Development Environment

### `rustup`

We are currently working with our toolchain and components pinned. These
constraints are defined in [./rust-toolchain](./rust-toolchain) and they should
be picked up by your rust development environment automatically.

### Editors

#### Doom Emacs (using lsp)

Install [direnv](https://direnv.net/).

Install rustup

```sh
curl https://sh.rustup.rs -sSf | sh
```

Edit your [~/.doom.d/init.el](~/.doom.d/init.el), uncommenting `rust`, and
adding the `lsp` option:

```emacs-lisp
(rust +lsp)
```

Install rustup dev-dependencies

```sh
rustup component add rls rust-analysis rust-src clippy-preview rustfmt-preview
```

