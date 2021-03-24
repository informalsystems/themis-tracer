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
    - [Viewing logical `unit`s](#viewing-logical-units)
        - [`list` all the units in the current context](#list-all-the-units-in-the-current-context)
            - [`unit list --fmt json`](#unit-list---fmt-json)
            - [`unit list --fmt json`](#unit-list---fmt-json-1)
        - [`show` all information about a particular unit](#show-all-information-about-a-particular-unit)
            - [`unit show --format json`](#unit-show---format-json)
            - [`unit show --format csv`](#unit-show---format-csv)
    - [`sync`ing repos in the context](#syncing-repos-in-the-context)
    - [`parse`ing specs](#parseing-specs)
        - [`parse --format json` (the default, if no argument is given)](#parse---format-json-the-default-if-no-argument-is-given)
        - [`parse --format csv`](#parse---format-csv)
    - [`linkify`ing spec files](#linkifying-spec-files)
    - [Cleanup](#cleanup)

<!-- markdown-toc end -->

## Setting the environment

These variables are used in the environment of the following tests. You can
ignore these when consulting this document for usage.

Where you see `$CMD` in the following you should just use the installed binary:
`themis-tracer`.

<!-- TODO replace by adding the executable to the path -->
<!-- $MDX set-CMD=../target/debug/themis-tracer,set-TRACER_HOME=../target/test-sandbox,set-RUST_LOG=error -->
```sh
$ echo CMD: $CMD
CMD: ../target/debug/themis-tracer
$ echo TRACER_HOME: $TRACER_HOME
TRACER_HOME: ../target/test-sandbox
```

## Show the current version

```sh
$ $CMD --version
tracer 0.1.0
```

## `init`ialize the tool

```sh
$ $CMD init
Initialized into ../target/test-sandbox/.tracer
$ ls $TRACER_HOME/.tracer
tracer.db
```

## Set the log level

Set the logging level by setting the environment variable `RUST_LOG`. Valid
levels are

- `error`
- `warn` (default)
- `info`
- `debug`
- `trace`

E.g.,

```sh
$ RUST_LOG=info $CMD --version 2>&1 | sed "s/^.*Z /[/"
[INFO  themis_tracer] log level set to info
tracer 0.1.0
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
* bar
  foo
```

Note that creating a new context also activates it. Thus the last created
context, `bar` is now active.

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

We'll also add a fake upstream to one of the repos:

```sh
$ cd repos/repo-a ; git remote add upstream git@github.com:informalsystems/themis-tracer.git
$ cd repos/repo-a ; git remote -v
upstream	git@github.com:informalsystems/themis-tracer.git (fetch)
upstream	git@github.com:informalsystems/themis-tracer.git (push)
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
> : A unit in a nested directory.\
> \
> |FLIM.1::FLAM.1| \
> : Second unit in the same directory. \
>   This one has a newline.  And refers to [FLIM.1]\
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
FLIM.1          repos/repo-a  A unit in a nested directory.
FLIM.1::FLAM.1  repos/repo-a  Second unit in the same directory. This one has a newline. And refers to [FLIM.1]
FOO.1           repos/repo-a  First unit.
FOO.1::BAR.1    repos/repo-a  A unit with a long description: “Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics).”
```

#### `unit list --fmt json`

Using the `--fmt json` option you can output the complete data of all logical
units in the context, serialized into json:

```sh
$ $CMD unit list --format json | sed "s:$(pwd)/::"
{"id":"FLIM.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":"git@github.com:informalsystems/themis-tracer.git","branch":null}}}},"file":"dir/spec-2.md","line":null,"content":"A unit in a nested directory.","references":[]}
{"id":"FLIM.1::FLAM.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":"git@github.com:informalsystems/themis-tracer.git","branch":null}}}},"file":"dir/spec-2.md","line":null,"content":"Second unit in the same directory. This one has a newline. And refers to [FLIM.1]","references":[]}
{"id":"FOO.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":"git@github.com:informalsystems/themis-tracer.git","branch":null}}}},"file":"spec-1.md","line":null,"content":"First unit.","references":[]}
{"id":"FOO.1::BAR.1","kind":"Requirement","repo":{"location":{"inner":{"Local":{"path":"repos/repo-a","upstream":"git@github.com:informalsystems/themis-tracer.git","branch":null}}}},"file":"spec-1.md","line":null,"content":"A unit with a long description: “Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics).”","references":[]}
```

#### `unit list --fmt json`

Using the `--fmt csv` option you can output the complete data of all logical
units in the context, serialized into csv:

```sh
$ $CMD unit list --format csv | sed "s:$(pwd)/::"
FLIM.1,Requirement,repos/repo-a,git@github.com:informalsystems/themis-tracer.git,,dir/spec-2.md,,A unit in a nested directory.
FLIM.1::FLAM.1,Requirement,repos/repo-a,git@github.com:informalsystems/themis-tracer.git,,dir/spec-2.md,,Second unit in the same directory. This one has a newline. And refers to [FLIM.1]
FOO.1,Requirement,repos/repo-a,git@github.com:informalsystems/themis-tracer.git,,spec-1.md,,First unit.
FOO.1::BAR.1,Requirement,repos/repo-a,git@github.com:informalsystems/themis-tracer.git,,spec-1.md,,"A unit with a long description: “Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics).”"
```

### `show` all information about a particular unit

Show all recorded information associated with the particular unit identified by
the given `TAG`:

```sh
$ $CMD unit show FOO.1::BAR.1 | sed "s:$(pwd)/::"
tag:   FOO.1::BAR.1
kind:  Requirement
repo:  repos/repo-a
file:  spec-1.md
line:
refs:

A unit with a long description: “Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics).”
```

#### `unit show --format json`

Using the `--format json` option outputs the complete data of a logical unit
serialized into JSON:

```sh
$ $CMD unit show FOO.1::BAR.1 --format json | sed "s:$(pwd)/::" | jq
{
  "id": "FOO.1::BAR.1",
  "kind": "Requirement",
  "repo": {
    "location": {
      "inner": {
        "Local": {
          "path": "repos/repo-a",
          "upstream": "git@github.com:informalsystems/themis-tracer.git",
          "branch": null
        }
      }
    }
  },
  "file": "spec-1.md",
  "line": null,
  "content": "A unit with a long description: “Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics).”",
  "references": []
}
```

#### `unit show --format csv`

Using the `--format csv` option outputs the complete data of a logical unit,
serialized into CSV:

```sh
$ $CMD unit show FOO.1::BAR.1 --format csv | sed "s:$(pwd)/::"
FOO.1::BAR.1,Requirement,repos/repo-a,git@github.com:informalsystems/themis-tracer.git,,spec-1.md,,"A unit with a long description: “Proofs, from the formal standpoint, are likewise nothing but finite series of formulae (with certain specifiable characteristics).”"
```

## `sync`ing repos in the context

When the artifacts in a repository have been changed, the database of logical
units is updated using the `sync` subcommand.

Let's make some changes to `spec-1.md` in `repo-a`:

```sh
$ cat > repos/repo-a/spec-1.md<<EOF \
> |FOO.2| \
> : We've updated the first unit. \
> \
> |FOO.2::BAZ.1| \
> : And we replaced FOO.1::BAR.1 with this unit. \
> EOF
```

After syncing, the units in the context will be updated accordingly:

```sh
$ $CMD sync
$ $CMD unit list | sed "s:$(pwd)/::"
FLIM.1          repos/repo-a  A unit in a nested directory.
FLIM.1::FLAM.1  repos/repo-a  Second unit in the same directory. This one has a newline. And refers to [FLIM.1]
FOO.2           repos/repo-a  We’ve updated the first unit.
FOO.2::BAZ.1    repos/repo-a  And we replaced FOO.1::BAR.1 with this unit.
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
  "content": "We can parse a file of logical units into different formats, preserving all critical content of the logical unit content.",
  "references": []
}
{
  "id": "PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1",
  "kind": "Requirement",
  "repo": null,
  "file": "parsing-spec.md",
  "line": null,
  "content": "The content of logical units must be preserved.\n\nEven when it spans multiple paragraphs.\n\n* Or\n* includes\n* lists",
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
  "content": "The folowing inline styling must be preserved:\n\n* **Strong** (**both** ways)\n* *Emphasizes* (*both* ways)\n* ~~Strikethrough~~\n* `code`\n* [links](/url)\n* ![images](/url \"fig:\")\n* smallcaps",
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
PARSE-SPECS.1,Requirement,,parsing-spec.md,,"We can parse a file of logical units into different formats, preserving all critical content of the logical unit content."
PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1,Requirement,,parsing-spec.md,,"The content of logical units must be preserved.

Even when it spans multiple paragraphs.

* Or
* includes
* lists"
PARSE-SPECS.1::CSV.1,Requirement,,parsing-spec.md,,Must support parse a file of specs into CSV.
PARSE-SPECS.1::INLINE.1,Requirement,,parsing-spec.md,,"The folowing inline styling must be preserved:

* **Strong** (**both** ways)
* *Emphasizes* (*both* ways)
* ~~Strikethrough~~
* `code`
* [links](/url)
* ![images](/url ""fig:"")
* smallcaps"
PARSE-SPECS.1::JSON.1,Requirement,,parsing-spec.md,,Must support parsing a file of specs into JSON.
```

## `linkify`ing spec files

The tool can add unit reference links and unit definition anchors to
specifications written in markdown.

Consider the specs used in the section on [managing
repositories](#managing-repositories), in `repo-a`, which is registered  in
context `foo`:

```sh
$ $CMD context list
  bar
* foo
$ $CMD repo list | sed "s:$(pwd)/::"
  repos/repo-a
  repos/repo-b
$ ls repos/repo-a/*.md repos/repo-a/dir/*.md
repos/repo-a/dir/spec-2.md
repos/repo-a/spec-1.md
```

We can linkify them with

```sh
$ $CMD linkify repos/repo-a/*.md repos/repo-a/dir/*.md
```

Which will change the files in place, yielding the following:

```sh
$ for f in repos/repo-a/*.md repos/repo-a/dir/*.md; do printf "\nin $f...\n\n"; cat $f | sed "s:$(pwd)/::"; done

in repos/repo-a/spec-1.md...

<span id="FOO.2">|FOO.2|</span>
:   We've updated the first unit.

<span id="FOO.2::BAZ.1">|FOO.2::BAZ.1|</span>
:   And we replaced FOO.1::BAR.1 with this unit.

in repos/repo-a/dir/spec-2.md...

<span id="FLIM.1">|FLIM.1|</span>
:   A unit in a nested directory.

<span id="FLIM.1::FLAM.1">|FLIM.1::FLAM.1|</span>
:   Second unit in the same directory. This one has a newline. And
    refers to [FLIM.1]

  [FLIM.1]: https://github.com/informalsystems/themis-tracer/blob/master/dir/spec-2.md#FLIM.1
```

<!-- FIXME: Remove need for this -->
## Cleanup

```sh
$ rm -rf ../target/test-sandbox
$ rm -rf ./repos
```
