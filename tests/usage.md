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
$ $CMD repo add repos/repo-a
$ $CMD repo add repos/repo-b
```

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

### `parse`ing specs

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

#### `parse --format json` (the default, if no argument is given)

The default formatting for parsed files is a stream of JSON objects:

```sh
$ $CMD parse parsing-spec.md | jq
{
  "id": "PARSE-SPECS.1",
  "kind": "Requirement",
  "source_file": "parsing-spec.md",
  "content": "We can parse a file of logical units into different formats, preserving all\ncritical content of the logical unit content.",
  "line": null,
  "column": null
}
{
  "id": "PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1",
  "kind": "Requirement",
  "source_file": "parsing-spec.md",
  "content": "The content of logical units must be preserved.\n\nEven when it spans multiple paragraphs.\n\n- Or\n- includes\n- lists",
  "line": null,
  "column": null
}
{
  "id": "PARSE-SPECS.1::CSV.1",
  "kind": "Requirement",
  "source_file": "parsing-spec.md",
  "content": "Must support parse a file of specs into CSV.",
  "line": null,
  "column": null
}
{
  "id": "PARSE-SPECS.1::INLINE.1",
  "kind": "Requirement",
  "source_file": "parsing-spec.md",
  "content": "The folowing inline styling must be preserved:\n\n- **Strong** (**both** ways)\n- *Emphasizes* (*both* ways)\n- ~~Strikethrough~~\n- `code`\n- [links](/url)\n- ![images](/url)\n- [smallcaps]{.smallcaps}",
  "line": null,
  "column": null
}
{
  "id": "PARSE-SPECS.1::JSON.1",
  "kind": "Requirement",
  "source_file": "parsing-spec.md",
  "content": "Must support parsing a file of specs into JSON.",
  "line": null,
  "column": null
}
```

#### `parse --format csv`

```sh
$ $CMD parse parsing-spec.md --format csv
id,kind,source_file,content,line,column
PARSE-SPECS.1,Requirement,parsing-spec.md,"We can parse a file of logical units into different formats, preserving all
critical content of the logical unit content.",,
PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1,Requirement,parsing-spec.md,"The content of logical units must be preserved.

Even when it spans multiple paragraphs.

- Or
- includes
- lists",,
PARSE-SPECS.1::CSV.1,Requirement,parsing-spec.md,Must support parse a file of specs into CSV.,,
PARSE-SPECS.1::INLINE.1,Requirement,parsing-spec.md,"The folowing inline styling must be preserved:

- **Strong** (**both** ways)
- *Emphasizes* (*both* ways)
- ~~Strikethrough~~
- `code`
- [links](/url)
- ![images](/url)
- [smallcaps]{.smallcaps}",,
PARSE-SPECS.1::JSON.1,Requirement,parsing-spec.md,Must support parsing a file of specs into JSON.,,
```

<!-- FIXME: Remove need for this -->
## Cleanup

```sh
$ rm -rf ../target/test-sandbox
$ rm -rf ./repos
```
