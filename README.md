# Themis Tracer

Themis Tracer is a tool to help provide requirements traceability for [Informal
Systems'][informal] verification-driven development (VDD) process. We intend on
tracing *logical units* (chunks of functionality) through:

* Human-language specifications (written in a slightly extended flavor of
  Markdown)
* Formal specifications (e.g. in TLA+)
* Code (initially only in Rust)

## Installation

Themis Tracer is written in [Rust][rust]. To install it, make sure you have the
latest version of the **stable** Rust toolchain installed and run the following:

```bash
# Clone this repository
> git clone https://github.com/informalsystems/themis-tracer/
> cd themis-tracer
# Install using Cargo
> cargo install
```

## Tutorial

### Step 1: Write your human-language specification

We make use of a variant of [PHP Markdown Extra's definition
lists][phpme-deflist] to define *logical units* (i.e. requirements) in Markdown.

```markdown
# Specification

|SPEC-HELLO.1|
:   When executed, the program must print out the text "Hello world!"
```

Logical units have unique identifiers associated with them. In this overly
simple case, we have the requirement labelled `SPEC-HELLO.1`. In order to
differentiate logical unit identifiers from ordinary [definition list
items][phpme-deflist], we enclose the tag in pipe symbols (`|`).

Each logical unit identifier must have a version number associated with it. In
this case, unit `SPEC-HELLO` has a version of `1` at present. This helps us
ensure that, when specifications change, we can automatically see which parts of
the code need to change too.

### Step 2: Implement your specification

Here we have a very basic Rust application that implements our specification.

```rust
//!
//! Our main application.
//! 
//! The "Implements" heading and list below is really important. It indicates
//! that this entire file is responsible for the implementation of all of the
//! logical units listed.
//!
//! # Implements
//!
//! * [SPEC-HELLO.1]
//!

fn main() {
    println!("Hello world!");
}
```

### Step 3: Tell Themis Tracer where all your repositories are

To tell Themis Tracer where to find all of your specifications and code, create
a file `.tracer.dhall` or `.tracer/package.dhall` (we use [Dhall][dhall] for
configuration) wherever you consider to be the **entrypoint** for your
collection of repositories:

```dhall
{- .tracer.dhall

   This particular configuration file shows an example of how to define the
   **entrypoint** for your collection of repositories.
-}

-- Import the Themis Tracer types
let Tracer = https://raw.githubusercontent.com/informalsystems/themis-tracer/master/config/package.dhall

let project: Tracer.Project =
    {
        {- A human-readable, descriptive, short name for the project -}
        name = "Hello World",
        {- The components that make up the project -}
        components = [
            {
                {- A human-readable, descriptive, short name for this component -}
                name = "Specifications",
                {- Here we can specify a "git://" (SSH) URL or an HTTPS URL as a source -}
                source = "git://github.com:informalsystems/themis-tracer#31352cc9977cc6e85444de6a1609b8315cab393d",
                {- If we only want to process specific files in the source -}
                path = "/examples/helloworld/**/*.md"
            },
            {
                name = "Rust implementation",
                source = "git://github.com/informalsystems/themis-tracer#31352cc9977cc6e85444de6a1609b8315cab393d",
                path = "/examples/helloworld/**/*.rs",
            }
        ]
    }

-- Expose the project object
in project
```

This is great for configuring remote repositories associated with your project,
but what about locally sourced versions of the repositories? For that you need
**local configuration**. Create a file `~/.themis/tracer/repos.dhall`:

```dhall
{- repos.dhall

   Configuration mapping remote repositories to local ones.
-}

let Tracer = https://raw.githubusercontent.com/informalsystems/themis-tracer/master/config/package.dhall

let repoMappings: List Tracer.RepositoryMapping =
    [
        {
            {- A globally unique ID to associate with this repository to allow
               for quick and easy reference -}
            id = "themis-tracer",
            remote = "git://github.com:informalsystems/themis-tracer",
            local = "/Users/manderson/work/themis-tracer"
        }
    ]

-- Expose the mappings
in repoMappings
```

If the source repository for a component **is** the repository in which the
`.tracer.dhall` configuration file is located (automatically detected by Themis
Tracer), then no mapping is needed.

TODO: Define CLI for managing repository mappings.

### Step 4: Run Themis Tracer to check that your spec is implemented

Once you've installed Themis Tracer, you can simply run it from the same folder
where you've created your `.tracer.dhall` configuration file:

```bash
> themis-tracer
✅ Specification is fully implemented and up-to-date!
```

### Step 5: Update your specification

To test how Themis Tracer reports differences between your specifications and
code, modify your specification as follows:

```markdown
# Specification

|SPEC-INPUT.1|
:   When executed, the program must print the text: "Hello! What's your name?",
    and allow the user to input their name.

|SPEC-HELLO.2|
:   Once the user's name has been obtained, the program must print out the text
    "Hello {name}!", where `{name}` must be replaced by the name obtained in
    [SPEC-INPUT.1].
```

In the above specification, we've introduced a new requirement (`SPEC-INPUT.1`),
and we've updated the version of an existing requirement (`SPEC-HELLO.1` became
`SPEC-HELLO.2`). If we run Themis Tracer again we'll get:

```bash
> themis-tracer
⚠️ SPEC-INPUT has not yet been implemented.
⚠️ SPEC-HELLO is out of date (version 1 has been implemented, but version 2 has been specified).
```

### Step 6: Incorporating defects

Themis Tracer also supports defect detection (GitHub only at present). It
connects to the publicly accessible API for your project's repositories and
scans for issues with the label `defect`. It then parses the subject and body of
the issue to see which logical units are currently defective.

If you have specified your `code` repositories' sources as GitHub, Themis Tracer
will automatically pick this up and try to connect to GitHub to detect any open
defects:

```bash
> themis-tracer
Specification is fully implemented and up-to-date, but the following logical
units have defects at present:

💔 SPEC-INPUT.1:
   - https://github.com/informalsystems/themis-tracer/issues/1

💔 SPEC-HELLO.2:
   - https://github.com/informalsystems/themis-tracer/issues/2
```

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
