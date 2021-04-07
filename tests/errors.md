# Errors

This document contains integration test which verify and illustrate how the tool
behaves under various error conditions.

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Errors](#errors)
    - [The environment](#the-environment)
    - [User CLI errors](#user-cli-errors)
    - [`context` errors](#context-errors)
        - [Adding a `repo` when there's no working context](#adding-a-repo-when-theres-no-working-context)
        - [Creating redundant `context`s](#creating-redundant-contexts)
        - [`switch` to a non-existent context](#switch-to-a-non-existent-context)
    - [`repo` errors](#repo-errors)
        - [Adding redundant repos to the context `repo`](#adding-redundant-repos-to-the-context-repo)
    - [Logical `unit`s](#logical-units)
        - [Ensure logical units are only reported in the respective context](#ensure-logical-units-are-only-reported-in-the-respective-context)
    - [`linkify`](#linkify)
        - [A warning is reported for invalid link references](#a-warning-is-reported-for-invalid-link-references)
        - [linkification is idempotent](#linkification-is-idempotent)
    - [`graph`](#graph)

<!-- markdown-toc end -->

## The environment

```sh
$ echo TRACER_HOME=$TRACER_HOME
TRACER_HOME=../target/test-sandbox
```

Some repos to work with

```sh
$ mkdir repos
$ git init repos/repo-a | sed "s:$(pwd)/::"
Initialized empty Git repository in repos/repo-a/.git/
$ git init repos/repo-b | sed "s:$(pwd)/::" # We trim the absolute path prefix, for testing purposes
Initialized empty Git repository in repos/repo-b/.git/
```

One of these repos has a remote:

```sh
$ git init --bare repos/repo-a-remote.git | sed "s:$(pwd)/::"
Initialized empty Git repository in repos/repo-a-remote.git/
$ cd repos/repo-a ; git remote add upstream ../repo-a-remote.git
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
$ kontxt unsupported-arg
error: Found argument 'unsupported-arg' which wasn't expected, or isn't valid in this context

USAGE:
    kontxt <SUBCOMMAND>

For more information try --help
[1]
```

## `context` errors

### Adding a `repo` when there's no working context

```sh
$ kontxt repo add repos/repo-a
Initialized into ../target/test-sandbox/.tracer
Error: No context is set. Try: `context switch <context>`
[1]
```

### Creating redundant `context`s

```sh
$ kontxt new foo
$ kontxt list
* foo
$ kontxt new foo
Error: A context named foo already exists
[1]
$ kontxt list
* foo
```

### `switch` to a non-existent context

```sh
$ kontxt switch nonexistent
Error: Context nonexistent does not exists
[1]
```

## `repo` errors

### Adding redundant repos to the context `repo`

```sh
$ kontxt switch foo
$ kontxt repo add repos/repo-a
$ kontxt repo add repos/repo-a 2>&1 | sed "s:$(pwd)/::"
Error: The repo repos/repo-a is already registered in the current context
```

## Logical `unit`s

### Ensure logical units are only reported in the respective context

The preceding has left us with the current working context, with its registered
repo and logical units:

```sh
$ kontxt list
* foo
$ kontxt repo list | sed "s:$(pwd)/::"
  repos/repo-a
$ kontxt unit list | sed "s:$(pwd)/::"
FOO.1         repos/repo-a  First unit.
FOO.1::BAR.1  repos/repo-a  Second unit.
```

We should be able to add `repos/repo-b` to a new context, and have only those
units belonging to that repo listed in the new context:

```sh
$ kontxt new bar
$ kontxt switch bar
$ kontxt repo add repos/repo-b
$ kontxt unit list | sed "s:$(pwd)/::"
FLIM.1          repos/repo-b  A unit in different repo.
FLIM.1::FLAM.1  repos/repo-b  A second unit in the same repo.
```

And these newly added units should not be added to the previous context

```sh
$ kontxt switch foo
$ kontxt unit list | sed "s:$(pwd)/::" 
FOO.1         repos/repo-a  First unit.
FOO.1::BAR.1  repos/repo-a  Second unit.
```

If we had a duplicate unit to a repo, the duplication is reported on `sync`:

```sh
$ cat > repos/repo-a/spec-dup.md<<EOF \
> |FOO.1| \
> : Duplicate unit. \
> EOF
$ kontxt sync 2>&1 | sed "s:$(pwd)/::g"
Error: Duplicate logical units found LOGICAL-UNIT{repo: repos/repo-a, file: spec-1.md, id: FOO.1, kind: Requirement, content: "First unit."} LOGICAL-UNIT{repo: repos/repo-a, file: spec-dup.md, id: FOO.1, kind: Requirement, content: "Duplicate unit."}
$ rm repos/repo-a/spec-dup.md
```

## `linkify`

### A warning is reported for invalid link references

```sh
$ kontxt switch foo
$ cat > repos/repo-a/spec-with-invalid-reference.md<<EOF \
> |BLOPS.1| \
> : A Reference to an invalid logical unit: [NO-UNIT.1] \
> EOF
$ kontxt file linkify repos/repo-a/spec-with-invalid-reference.md
Error: linkifying file repos/repo-a/spec-with-invalid-reference.md

Caused by:
    0: linkifying string
    1: No unit found corresponding to tag NO-UNIT.1
[1]
```

### linkification is idempotent

```sh
$ kontxt file linkify repos/repo-a/spec-1.md
$ cp repos/repo-a/spec-1.md repos/repo-a/spec-1.md.copy
$ kontxt file linkify repos/repo-a/spec-1.md
$ cat repos/repo-a/spec-1.md
<span id="FOO.1">|FOO.1|</span>
:   First unit.

<span id="FOO.1::BAR.1">|FOO.1::BAR.1|</span>
:   Second unit.
$ diff repos/repo-a/spec-1.md repos/repo-a/spec-1.md.copy
```

## `graph`

An error is logged for any orphan units when graphing:

```sh
$ cat > repos/repo-a/spec-with-orphan-unit.md<<EOF \
> |PARENT.1::ORPHAN.1| \
> : This unit has no parent. \
> EOF
$ kontxt sync
$ RUST_LOG=warn kontxt generate graph --format dot 2>&1 | sed 's/^\[[^ ]* /[/'
[WARN  tracer::graph] orphan unit PARENT.1::ORPHAN.1 is missing its parent PARENT.1
digraph {
    0 [ label="BLOPS.1" tooltip="A Reference to an invalid logical unit: [NO-UNIT.1]" href="TODO#BLOPS.1" ]
    1 [ label="FOO.1" tooltip="First unit." href="TODO#FOO.1" ]
    2 [ label="FOO.1::BAR.1" tooltip="Second unit." href="TODO#FOO.1::BAR.1" ]
    3 [ label="PARENT.1::ORPHAN.1" tooltip="This unit has no parent." href="TODO#PARENT.1::ORPHAN.1" ]
    1 -> 2 [ ]
}

```
