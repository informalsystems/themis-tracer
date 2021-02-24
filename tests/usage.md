<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [-](#-)
    - [Show the current `version`](#show-the-current-version)
    - [`init` a new context](#init-a-new-context)
    - [`parse` specs](#parse-specs)
        - [Default [`--format json`]](#default---format-json)
        - [`--format csv`](#--format-csv)

<!-- markdown-toc end -->

## Setting the environment

<!-- TODO replace by adding the executable to the path -->
<!-- $MDX set-CMD=../target/debug/themis-tracer -->
```sh
$ echo CMD: $CMD
CMD: ../target/debug/themis-tracer
```

Where you see `$CMD` in the following you should just use the installed binary
`themis-tracer`.

### Show the current version

```sh
$ $CMD --version
whorl 0.1.0
```

### `init`ialize a new context

```sh
$ WHORL_HOME=/tmp $CMD init
Initialized whorl to /tmp/.whorl
$ ls /tmp/.whorl
contexts
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
