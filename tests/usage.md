# Usage

This document contains integration tests which also serve as succinct,
reference-friendly, documentation of the tool's usage.

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [-](#-)
    - [Show the current version](#show-the-current-version)
    - [`init`ialize the tool](#initialize-the-tool)
    - [`parse` specs](#parse-specs)
        - [Default [`--format json`]](#default---format-json)
        - [`--format csv`](#--format-csv)
    - [Manage `context`s](#manage-contexts)
        - [`context new`](#context-new)
        - [`context list`](#context-list)
        - [`context switch`](#context-switch)
    - [Cleanup](#cleanup)

<!-- markdown-toc end -->

## Setting the environment

<!-- TODO replace by adding the executable to the path -->
<!-- $MDX set-CMD=../target/debug/themis-tracer,set-TRACER_HOME=../target/test-sandbox -->
```sh
$ echo CMD: $CMD
CMD: ../target/debug/themis-tracer
$ echo TRACER_HOME: $TRACER_HOME
TRACER_HOME: ../target/test-sandbox
```

Where you see `$CMD` in the following you should just use the installed binary
`themis-tracer`.

### Show the current version

```sh
$ $CMD --version
tracer 0.1.0
```

### `init`ialize the tool

```sh
$ $CMD init
Initialized into ../target/test-sandbox/.tracer
$ ls $TRACER_HOME/.tracer
tracer.db
```

### `parse` specs

#### Default [`--format json`]

```sh
$ $CMD parse spec.md
{"id":"BAR.1::BAZ.2","kind":"Requirement","source_file":"spec.md","content":"Bloop drop.","line":null,"column":null}
{"id":"BIP.1::BOP.2","kind":"Requirement","source_file":"spec.md","content":"Floop droop drop plop.\nFloop droop drop plop.\nFloop droop drop plop.","line":null,"column":null}
{"id":"FOO.1","kind":"Requirement","source_file":"spec.md","content":"Bish bosh, flip flop.","line":null,"column":null}
{"id":"ZAP.1::ZING.2::ZOG.12","kind":"Requirement","source_file":"spec.md","content":"Floop droop drop plop.\nFloop droop drop plop.\nFloop droop drop plop.","line":null,"column":null}
```

#### `--format csv`

```sh
$ $CMD parse spec.md --format csv
id,kind,source_file,content,line,column
BAR.1::BAZ.2,Requirement,spec.md,Bloop drop.,,
BIP.1::BOP.2,Requirement,spec.md,"Floop droop drop plop.
Floop droop drop plop.
Floop droop drop plop.",,
FOO.1,Requirement,spec.md,"Bish bosh, flip flop.",,
ZAP.1::ZING.2::ZOG.12,Requirement,spec.md,"Floop droop drop plop.
Floop droop drop plop.
Floop droop drop plop.",,
```

## Manage `context`s

### `context new`

```sh
$ $CMD context new foo
$ $CMD context new bar
```

### `context list`

List the existing contexts as follows:

```sh
$ $CMD context list
  bar
  foo
```

### `context switch`

Switch contexts as follows:

```sh
$ $CMD context switch bar
```

The current context is indicted by a `*` preceding its name in the context list:

```sh
$ $CMD context list
* bar
  foo
$ $CMD context switch foo
$ $CMD context list
  bar
* foo
```

## managing repositories

Assume we want to work with the following repositories:

```sh
$ mkdir repos
$ git init repos/repo-a | sed "s:$(pwd)/::"
Initialized empty Git repository in repos/repo-a/.git/
$ git init repos/repo-b | sed "s:$(pwd)/::" # We trim the absolute path prefix, for testing purposes
Initialized empty Git repository in repos/repo-b/.git/
```

**NOTE**: Here and following, we use the filter `| sed "s:$(pwd)/::"` to trim
the absolute path prefix from output, so that the accuracy of this documentation
is ensured by integration tests. However, the tool always associates
repositories with their absolute path. This is the unique name of a repository
(in the user's local file system or on the world wide web).

### `add` repos to a context

```sh
$ $CMD add repos/repo-a
$ $CMD add repos/repo-b
```

### `list` the repos in the current context

```sh
$ $CMD context list
  bar
* foo
$ $CMD repos | sed "s:$(pwd)/::" # We trim the absolute path prefix, for testing purposes
  repos/repo-a
  repos/repo-b
```

Each context has it's own associated repos. We haven't added any repos to  the
context `bar` yet, so if we switch contexts and list its repos, we'll see that
reflected:

```sh
$ $CMD context switch bar
$ $CMD repos
```

<!-- FIXME: Remove need for this -->
## Cleanup

```sh
$ rm -rf ../target/test-sandbox
$ rm -rf ./repos
```
