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
spec.md FOO.1 Requirement <Bish bosh.>
spec.md BAR.1::BAZ.2 Requirement <Bloop drop.>
```
