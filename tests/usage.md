# Usage

This document contains integration tests which also serve as succinct,
reference-friendly, documentation of the tool's usage.

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Usage](#usage)
    - [Setting the environment](#setting-the-environment)
        - [Show the current version](#show-the-current-version)
        - [`init`ialize the tool](#initialize-the-tool)
    - [Manage `context`s](#manage-contexts)
        - [`context new`](#context-new)
        - [`context list`](#context-list)
        - [`context switch`](#context-switch)
    - [managing `repo`sitories](#managing-repositories)
        - [`add` repos to the current working context](#add-repos-to-the-current-working-context)
        - [`list` the repos in the current context](#list-the-repos-in-the-current-context)
    - [Viewing logical `units`](#viewing-logical-units)
    - [`parse`ing specs](#parseing-specs)
        - [`parse --format json` (the default, if no argument is given)](#parse---format-json-the-default-if-no-argument-is-given)
        - [`parse --format csv`](#parse---format-csv)
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

## managing `repo`sitories

Assume we want to work with the following repositories:

```sh
$ mkdir repos
$ git init repos/repo-a | sed "s:$(pwd)/::"
Initialized empty Git repository in repos/repo-a/.git/
$ git init repos/repo-b | sed "s:$(pwd)/::" # We trim the absolute path prefix, for testing purposes
Initialized empty Git repository in repos/repo-b/.git/
```

Assume also that `repo-a` contains some specs with logical units:

```sh
$ cat > repos/repo-a/spec-1.md<<EOF \
> |FOO.1| \
> : First unit. \
> \
> |FOO.1::BAR.1| \
> : A unit with a long description: "Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics)." \
> EOF
$ mkdir repos/repo-a/dir
$ cat > repos/repo-a/dir/spec-2.md <<EOF \
> |FLIM.1| \
> : A unit in a nested directory. \
> \
> |FLIM.1::FLAM.1| \
> : Second unit in the same directory. \
> EOF
```

**NOTE**: Here and following, we use the filter `| sed "s:$(pwd)/::"` to trim
the absolute path prefix from output, so that the accuracy of this documentation
is ensured by integration tests. However, the tool always associates
repositories with their absolute path. This is the unique name of a repository
(in the user's local file system or on the world wide web).

### `add` repos to the current working context

Add a repo to your current working context as follows:

```sh
$ $CMD repo add repos/repo-a
$ $CMD repo add repos/repo-b
```

When a repository is added to a context, all of the logical units that the tool
can find in the repository are loaded into the database. See [Viewing logical
units](#viewing-logical-units).

### `list` the repos in the current context

```sh
$ $CMD context list
  bar
* foo
$ $CMD repo list | sed "s:$(pwd)/::" # We trim the absolute path prefix, for testing purposes
  repos/repo-a
  repos/repo-b
```

Each context has it's own associated repos. We haven't added any repos to  the
context `bar` yet, so if we switch contexts and list its repos, we'll see that
reflected:

```sh
$ $CMD context switch bar
$ $CMD repo list
```

## Viewing logical `unit`s

### `list` all the units in the current context

`unit list` outputs a human readable synopsis of all units in the current context:

```sh
$ $CMD context switch foo
$ $CMD context list
  bar
* foo
$ $CMD unit list | sed "s:$(pwd)/::" # We trim the absolute path prefix, for testing purposes
FLIM.1          A unit in a nested directory.                                                                            repos/repo-a
FLIM.1::FLAM.1  Second unit in the same directory.                                                                       repos/repo-a
FOO.1           First unit.                                                                                              repos/repo-a
FOO.1::BAR.1    A unit with a long description: "Proofs, from the formal standpoint, are likewise nothing but finite...  repos/repo-a
```

#### `unit list --fmt json`

Using the `--fmt json` option you can output the complete data of all logical
units in the context, serialized into json:

```sh
$ $CMD unit list --format json | sed "s:$(pwd)/::"
{"id":"FLIM.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":null,"branch":null}}}},"file":"dir/spec-2.md","line":null,"content":"A unit in a nested directory.","references":[]}
{"id":"FLIM.1::FLAM.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":null,"branch":null}}}},"file":"dir/spec-2.md","line":null,"content":"Second unit in the same directory.","references":[]}
{"id":"FOO.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":null,"branch":null}}}},"file":"spec-1.md","line":null,"content":"First unit.","references":[]}
{"id":"FOO.1::BAR.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":null,"branch":null}}}},"file":"spec-1.md","line":null,"content":"A unit with a long description: \"Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics).\"","references":[]}
```

#### `unit list --fmt json`

Using the `--fmt csv` option you can output the complete data of all logical
units in the context, serialized into csv:

```sh
$ $CMD unit list --format csv | sed "s:$(pwd)/::"
FLIM.1,Requirement,repos/repo-a,,,dir/spec-2.md,,A unit in a nested directory.
FLIM.1::FLAM.1,Requirement,repos/repo-a,,,dir/spec-2.md,,Second unit in the same directory.
FOO.1,Requirement,repos/repo-a,,,spec-1.md,,First unit.
FOO.1::BAR.1,Requirement,repos/repo-a,,,spec-1.md,,"A unit with a long description: ""Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics)."""
```

## `parse`ing specs

You can use the tool to parse logical units out of individual files, so that you
can do computations with the specs via your own scripts or programs.

The following spec describes our current support for parsing logical units:

<!-- $MDX file=parsing-spec.md -->
```markdown
# Parsing logical units

|PARSE-SPECS.1|
: We can parse a file of logical units into different formats, preserving all
  critical content of the logical unit content.

## Serialization Format

Supported formats include:

|PARSE-SPECS.1::JSON.1|
: Must support parsing a file of specs into JSON.

|PARSE-SPECS.1::CSV.1|
: Must support parse a file of specs into CSV.

## Content

|PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1|
: The content of logical units must be preserved.
: Even when it spans multiple paragraphs.
: - Or
 - includes
 - lists

|PARSE-SPECS.1::INLINE.1|
: The folowing inline styling must be preserved:
: - **Strong** (__both__ ways)
 - *Emphasizes* (_both_ ways)
 - ~~Strikethrough~~
 - `code`
 - [links](/url)
 - ![images](/url)
 - [smallcaps]{.smallcaps}
```

We'll use this spec as an example to illustrate the options for parsing logical
units.

<!-- TODO Annotate with verification tags, tying to the implementations -->

### `parse --format json` (the default, if no argument is given)

The default formatting for parsed files is a stream of JSON objects:

```sh
$ $CMD parse parsing-spec.md | jq
{
  "id": "PARSE-SPECS.1",
  "kind": "Requirement",
  "repo": null,
  "file": "parsing-spec.md",
  "line": null,
  "content": "We can parse a file of logical units into different formats, preserving all\ncritical content of the logical unit content.",
  "references": []
}
{
  "id": "PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1",
  "kind": "Requirement",
  "repo": null,
  "file": "parsing-spec.md",
  "line": null,
  "content": "The content of logical units must be preserved.\n\nEven when it spans multiple paragraphs.\n\n- Or\n- includes\n- lists",
  "references": []
}
{
  "id": "PARSE-SPECS.1::CSV.1",
  "kind": "Requirement",
  "repo": null,
  "file": "parsing-spec.md",
  "line": null,
  "content": "Must support parse a file of specs into CSV.",
  "references": []
}
{
  "id": "PARSE-SPECS.1::INLINE.1",
  "kind": "Requirement",
  "repo": null,
  "file": "parsing-spec.md",
  "line": null,
  "content": "The folowing inline styling must be preserved:\n\n- **Strong** (**both** ways)\n- *Emphasizes* (*both* ways)\n- ~~Strikethrough~~\n- `code`\n- [links](/url)\n- ![images](/url)\n- [smallcaps]{.smallcaps}",
  "references": []
}
{
  "id": "PARSE-SPECS.1::JSON.1",
  "kind": "Requirement",
  "repo": null,
  "file": "parsing-spec.md",
  "line": null,
  "content": "Must support parsing a file of specs into JSON.",
  "references": []
}
```

### `parse --format csv`

```sh
$ $CMD parse parsing-spec.md --format csv
PARSE-SPECS.1,Requirement,,parsing-spec.md,,"We can parse a file of logical units into different formats, preserving all
critical content of the logical unit content."
PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1,Requirement,,parsing-spec.md,,"The content of logical units must be preserved.

Even when it spans multiple paragraphs.

- Or
- includes
- lists"
PARSE-SPECS.1::CSV.1,Requirement,,parsing-spec.md,,Must support parse a file of specs into CSV.
PARSE-SPECS.1::INLINE.1,Requirement,,parsing-spec.md,,"The folowing inline styling must be preserved:

- **Strong** (**both** ways)
- *Emphasizes* (*both* ways)
- ~~Strikethrough~~
- `code`
- [links](/url)
- ![images](/url)
- [smallcaps]{.smallcaps}"
PARSE-SPECS.1::JSON.1,Requirement,,parsing-spec.md,,Must support parsing a file of specs into JSON.
```

<!-- FIXME: Remove need for this -->
## Cleanup

```sh
$ rm -rf ../target/test-sandbox
$ rm -rf ./repos
```
