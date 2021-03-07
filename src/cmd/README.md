# The `cmd` submodule

The `cmd` submodule contains all the logic for CLI.

Two submodules in this directory are special:

- The `opt` module is responsible for the CLI parsing.
- The `format` module manages the serialization output formats used across
  a number of subcommands .

Each of the remaining modules exports a `run` function used to execute the
subcommand corresponding the module's name.
