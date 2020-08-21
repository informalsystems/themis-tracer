## Setting the environment

<!-- $MDX set-CMD=../target/debug/themis-tracer -->
```sh
$ echo CMD: $CMD
CMD: ../target/debug/themis-tracer
```


### Can print version

```sh
$ $CMD --version
themis-tracer 0.1.0
```


### `parse`


```sh
$ $CMD parse spec.md
{"id":"FOO.1","kind":"Requirement","source_file":"spec.md","content":"Bish bosh, flip flop.","line":null,"column":null}
{"id":"BAR.1::BAZ.2","kind":"Requirement","source_file":"spec.md","content":"Bloop drop.","line":null,"column":null}
{"id":"BIP.1::BOP.2","kind":"Requirement","source_file":"spec.md","content":"Floop droop drop plop.\nFloop droop drop plop.\nFloop droop drop plop.","line":null,"column":null}
{"id":"ZAP.1::ZING.2::ZOG.12","kind":"Requirement","source_file":"spec.md","content":"Floop droop drop plop.\nFloop droop drop plop.\nFloop droop drop plop.","line":null,"column":null}
```
