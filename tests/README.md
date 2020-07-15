# Themis Tracer Tests

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->

**Table of Contents**

- [Themis Tracer Tests](#themis-tracer-tests)
  - [Synopsis](#synopsis)
    - [System dependencies](#system-dependencies)
    - [Running](#running)
    - [Accepting changes](#accepting-changes)
  - [Cram style tests](#cram-style-tests)

<!-- markdown-toc end -->

## Synopsis

### System dependencies

- [ocaml-mdx][]

[mdx]: https://github.com/realworldocaml/mdx#mdx----executable-code-blocks-inside-markdown-files

### Running

```sh skip
make test
```

### Accepting changes

```sh skip
make promote
```

## Cram style tests

Any `*.md` files in this directory can include embedded integration tests.

A cram style tests specifies a shell command and the expected output, following
the format

```sh skip
$ some command
expected output
```

If running `some command` doesn't produce `expected output`, the test fails with
a diff presenting the difference between what was expected and the "corrected"
version that actually resulted. If you corrected version is actually correct,
run

```sh skip
make promote
```

to accept the changes and update the tests.
