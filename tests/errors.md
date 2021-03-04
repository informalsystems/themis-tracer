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

### Creating redundant `context`s

```sh
$ $CMD context new foo
$ $CMD context list
  foo
$ $CMD context new foo
Error: A context named foo already exists
[1]
$ $CMD context list
  foo
```

### `switch` to a non-existent context

```sh
$ $CMD context switch nonexistent
Error: Context nonexistent does not exists
[1]
```

<!-- FIXME: Remove need for this -->
## Cleanup

```sh
$ rm -rf ../target/test-sandbox
```
