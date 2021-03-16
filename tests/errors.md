# Errors

This document contains integration test which verify and illustrate how the tool
behaves under various error conditions.

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Errors](#errors)
    - [Setting the environment](#setting-the-environment)
    - [User CLI errors](#user-cli-errors)
    - [`init`ialization errors](#initialization-errors)
        - [Redundant `init`ialization](#redundant-initialization)
    - [`context` errors](#context-errors)
        - [Creating redundant `context`s](#creating-redundant-contexts)
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

Some repos work with

```sh
$ mkdir repos
$ git init repos/repo-a | sed "s:$(pwd)/::"
Initialized empty Git repository in repos/repo-a/.git/
$ git init repos/repo-b | sed "s:$(pwd)/::" # We trim the absolute path prefix, for testing purposes
Initialized empty Git repository in repos/repo-b/.git/
```

And some specs in the repos:

```sh
$ cat > repos/repo-a/spec-1.md<<EOF \
> |FOO.1| \
> : First unit. \
> \
> |FOO.1::BAR.1| \
> : Second unit. \
> EOF
$ cat > repos/repo-b/spec-2.md <<EOF \
> |FLIM.1| \
> : A unit in different repo. \
> \
> |FLIM.1::FLAM.1| \
> : A second unit in the same repo. \
> EOF
```

## User CLI errors

```sh
$ $CMD unsupported-arg
error: Found argument 'unsupported-arg' which wasn't expected, or isn't valid in this context

USAGE:
    themis-tracer <SUBCOMMAND>

For more information try --help
[1]
```

## `init`ialization errors

### Redundant `init`ialization

```sh
$ $CMD init
Initialized into ../target/test-sandbox/.tracer
$ $CMD init
Error: Already initialized in ../target/test-sandbox/.tracer
[1]
```

## `context` errors

### Adding a `repo` when there's no working context

```sh
$ $CMD repo add repos/repo-a
Error: No context is set. Try: `context switch <context>`
[1]
```

### Creating redundant `context`s

```sh
$ $CMD context new foo
$ $CMD context list
* foo
$ $CMD context new foo
Error: A context named foo already exists
[1]
$ $CMD context list
* foo
```

### `switch` to a non-existent context

```sh
$ $CMD context switch nonexistent
Error: Context nonexistent does not exists
[1]
```

## `repo` errors

### Adding redundant repos to the context `repo`

```sh
$ $CMD context switch foo
$ $CMD repo add repos/repo-a
$ $CMD repo add repos/repo-a 2>&1 | sed "s:$(pwd)/::"
Error: The repo repos/repo-a is already registered in the current context
```

## Logical `unit`s

### Ensure logical units are only reported in the respective context

The preceding has left us with the current working context, with its registered
repo and logical units:

```sh
$ $CMD context list
* foo
$ $CMD repo list | sed "s:$(pwd)/::"
  repos/repo-a
$ $CMD unit list | sed "s:$(pwd)/::"
FOO.1         repos/repo-a  First unit.
FOO.1::BAR.1  repos/repo-a  Second unit.
```

We should be able to add `repos/repo-b` to a new context, and have only those
units belonging to that repo listed in the new context:

```sh
$ $CMD context new bar
$ $CMD context switch bar
$ $CMD repo add repos/repo-b
$ $CMD unit list | sed "s:$(pwd)/::"
FLIM.1          repos/repo-b  A unit in different repo.
FLIM.1::FLAM.1  repos/repo-b  A second unit in the same repo.
```

And these newly added units should not be added to the previous context

```sh
$ $CMD context switch foo
$ $CMD unit list | sed "s:$(pwd)/::" 
FOO.1         repos/repo-a  First unit.
FOO.1::BAR.1  repos/repo-a  Second unit.
```

<!-- FIXME: Remove need for this -->
## Cleanup

```sh
$ rm -rf ../target/test-sandbox
$ rm -rf ./repos
```
