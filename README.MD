# AWS SSM Helper
AWS SSM Parameter Store Helper - A command line helper for AWS SSM Parameters, written in Rust.

## Command Line
```
USAGE:
    ssm_helper [FLAGS] [OPTIONS] <SUBCOMMAND>
FLAGS:
    -d, --decrypt    Decrypt Parameter Value
    -h, --help       Prints help information
    -q, --quiet      Quiet Mode => Only Errors and Parameter Output
    -V, --version    Prints version information
OPTIONS:
    -r, --region <region>    AWS Region [default: us-east-1]
SUBCOMMANDS:
    clone       Copy Parameter's Value from origin key to destination key [aliases: c]
    get         Get Parameter by Name (or Path) [aliases: g]
    help        Prints this message or the help of the given subcommand(s)
    list-all    List All Parameters [aliases: la, all]
    template    Template - Substitute vars in <templatein> and write to <templateout> or STDOUT [aliases: t]
```

## TODO

    [ ] Implement:
    [ ] - Quiet
    [ ] - Template
    [ ] - Clone
    [ ] - Fail Crate
    [ ] - Default for Requests?

    [ ] Improve:
    [ ] - pagination calls