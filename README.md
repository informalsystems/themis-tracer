# Themis Tracer

Themis Tracer is a tool to help provide development contexts.

It is developed by [Informal Systems'][informal] to support Development
and Operations that are Verifiable and Explicitly Specified (DOVES🕊).

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->

**Table of Contents**

- [Themis Tracer](#themis-tracer)
  - [(Planned) Features](#planned-features)
    - [**WIP** Context management](#wip-context-management)
    - [**WIP** Tracing](#wip-tracing)
    - [**TODO** Tracking](#todo-tracking)
    - [**TODO** Monitoring](#todo-monitoring)
  - [Installation](#installation)
    - [Prerequisites](#prerequisites)
    - [From git using cargo](#from-git-using-cargo)
    - [From source](#from-source)
  - [Documentation](#documentation)
    - [Tutorial](#tutorial)
      - [Logical units in markdown](#logical-units-in-markdown)
      - [TODO](#todo)
  - [License](#license)

<!-- markdown-toc end -->

## (Planned) Features

### **WIP** Context management

Manage multiple parallel contexts, spread across any number of repositories.

- [x] Enable switching between contexts.
- [ ] Support nested contexts, providing different perspectives to empower
      different kinds of work on the same domain.
- [ ] An integrated HUD to show key terminology, specifications, and diagrams,
      to support focused work and effective communication without noisy
      backchannels.

### **WIP** Tracing

Trace _logical units_ (chunks of functionality) through:

- [x] Human-language specifications (written in a slightly extended flavor of
      Markdown)
- [ ] Formal specifications (e.g. in TLA+)
- [ ] Code (initially only in Rust)

### **TODO** Tracking

Automate the `specify -> formalize -> implement -> verify -> deploy -> revise`
life cycle, by tracking the flow of system properties from conception to
delivery and back again.

- [ ] Track the progress of logical units, getting a quick overview of
      units yet to be formalized/implemented/verified/deployed, and the level of
      progress towards implementation.

### **TODO** Monitoring

Verify coherence and consistency of your development with its context in CI

- [ ] Rationalize change management by catching unplanned changes to
      implementations, and flagging them for review by those responsible for
      implementation.
- [ ] Ensure implementations are kept up to date with changing specifications,
      by catching implementation units that get out of date with update
      specifications.

## Installation

The tool is currently in early development, so expect snags.

### Prerequisites

- [cargo](https://doc.rust-lang.org/book/ch01-01-installation.html#installation)
- [pandoc](https://pandoc.org/installing.html) (tested on pandoc >= 2.9)
- [sqlite3](https://www.sqlite.org/index.html) (tested on sqlite >= 3.33): You
  probably already have this on your system. Check with `sqlite3 --version`. If
  you need to install it, check you OS's package manager.

### From git using cargo

```sh
cargo install --git ssh://git@github.com/informalsystems/themis-tracer.git
```

If this fails for any reason, please [open a
ticket](https://github.com/informalsystems/themis-tracer/issues/new) and try
installing from source, as documented in the next section.

### From source

```sh
git clone git@github.com:informalsystems/themis-tracer.git
cd themis-tracer
cargo install
```

## Documentation

See the [CLI usage documentation](./tests/usage.md).

### Tutorial

#### Logical units in markdown

We make use of a variant of [PHP Markdown Extra's definition
lists][phpme-deflist] to define _logical units_ (i.e. requirements) in Markdown.

```markdown
# Specification

|SPEC-HELLO.1|
: When executed, the program must print out the text "Hello world!"
```

Logical units have unique identifiers associated with them. In this overly
simple case, we have the requirement labelled `SPEC-HELLO.1`. In order to
differentiate logical unit identifiers from ordinary [definition list
items][phpme-deflist], we enclose the tag in pipe symbols (`|`).

Each logical unit identifier must have a version number associated with it. In
this case, unit `SPEC-HELLO` has a version of `1` at present. This helps us
ensure that, when specifications change, we can automatically see which parts of
the code need to change too.

#### TODO

## License

Copyright 2020 Informal Systems Inc. and contributors (see CONTRIBUTORS.md)

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[informal]: https://informal.systems/
[rust]: https://www.rust-lang.org/
[phpme-deflist]: https://michelf.ca/projects/php-markdown/extra/#def-list
[dhall]: https://dhall-lang.org/
