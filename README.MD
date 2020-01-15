AWS SSM Helper
===
AWS SSM Parameter Store Helper - A command line tool for AWS SSM Parameters, written in Rust.

Build Status: ![](https://github.com/rflaperuta/ssm_helper/workflows/.github/workflows/rust.yml/badge.svg)

Contents
- [AWS SSM Helper](#aws-ssm-helper)
    - [About](#about)
    - [Installation](#installation)
    - [Command Line](#command-line)
    - [TODO](#todo)
    - [License](#license)
        - [Contribution](#contribution)

## About

No more saving credentials or configuration parameters in local files manualy.

Now you can organize all your Application Parameters inside AWS SSM Parameter Store, and retrieve them when needed.

You can also, generate config files on the fly, just by using embedded handlebars template processing.

More:

[AWS Systems Manager Parameter Store](https://aws.amazon.com/systems-manager/features/#Parameter_Store)

[AWS Systems Manager Parameters Documentation](https://docs.aws.amazon.com/systems-manager/latest/userguide/sysman-paramstore-about.html)


## Installation
Install Rust: [rust-lang](https://www.rust-lang.org/en-US/install.html)

For now:
```
git clone git@github.com:rflaperuta/ssm_helper.git
cd ./ssm_helper
cargo build --release
cd ./target/release/
strip ./ssm_helper
cp ./ssm_helper /your/bin/path
```

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
    Implement:
    [ ] Quiet Mode
    [ ] Logging
    [ ] Template Processing
    [ ] Clone Parameter Value
    [ ] Fail Crate
    [ ] Impl Default for Requests?

    Improve:
    [ ] Pagination calls
    [ ] Readme
    [ ] Output: human readable != json
    [ ] Tests

    Cargo Install:
    [ ] Docs
    [ ] CI/CD
    [ ] Badges

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.