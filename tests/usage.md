## Setting the environment

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
themis-tracer 0.1.0
```


### `parse` specs

#### Default [`--format json`]

```sh
$ $CMD parse spec.md
{"id":"FOO.1","kind":"Requirement","source_file":"spec.md","content":"Bish bosh, flip flop.","line":null,"column":null}
{"id":"BAR.1::BAZ.2","kind":"Requirement","source_file":"spec.md","content":"Bloop drop.","line":null,"column":null}
{"id":"BIP.1::BOP.2","kind":"Requirement","source_file":"spec.md","content":"Floop droop drop plop.\nFloop droop drop plop.\nFloop droop drop plop.","line":null,"column":null}
{"id":"ZAP.1::ZING.2::ZOG.12","kind":"Requirement","source_file":"spec.md","content":"Floop droop drop plop.\nFloop droop drop plop.\nFloop droop drop plop.","line":null,"column":null}
```

#### `--format csv`

```sh
$ $CMD parse spec.md --format csv
id,kind,source_file,content,line,column
FOO.1,Requirement,spec.md,"Bish bosh, flip flop.",,
BAR.1::BAZ.2,Requirement,spec.md,Bloop drop.,,
BIP.1::BOP.2,Requirement,spec.md,"Floop droop drop plop.
Floop droop drop plop.
Floop droop drop plop.",,
ZAP.1::ZING.2::ZOG.12,Requirement,spec.md,"Floop droop drop plop.
Floop droop drop plop.
Floop droop drop plop.",,
```
