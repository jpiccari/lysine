# lysine

> **Prevents the spread of commands if they ever get off the island**

## About

`lysine` kills off commands with the absense of strong intent for the command to continue running. If you
have a command that should run as long as some interaction is happening, `lysine` may help you. There are
few assumptions and the only requirement is that a file is modified every so often. Typically, this is done
using the `touch` command at regular intervals.


```plain
$ lysine --help
Prevents the spread of commands if they ever get off the island

Usage: lysine [OPTIONS] <FILE> <COMMAND>...

Arguments:
  <FILE>        Use file modified timestamp for lysine check
  <COMMAND>...  Command (and args) to run

Options:
  -m, --max-age <MAX_AGE>  Maximum age of command in seconds [default: 60]
  -h, --help               Print help
  -V, --version            Print version
```