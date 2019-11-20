use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ssm_helper",
    about = "AWS Parameter Store Helper - A command line helper for AWS SSM Parameters, written in Rust."
)]
pub struct Opt {
    /// AWS Region
    #[structopt(short = "r", long = "region", default_value = "us-east-1")]
    pub region: String,
    /// Decrypt Parameter Value
    #[structopt(short = "d", long = "decrypt")]
    pub decrypt: bool,
    /// Overwrite Destination Parameter
    #[structopt(short = "o", long = "overwrite")]
    pub overwrite: bool,
    /// Quiet Mode => Only Errors and Parameter Output
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// List All Parameters
    #[structopt(name = "list-all", visible_alias = "la")]
    ListAll,
    /// Get Parameter by Name (or Path)
    #[structopt(name = "get", visible_alias = "g")]
    Get {
        /// Parameter Name
        #[structopt(required = true, min_values = 1, max_values = 10)]
        name: Vec<String>,
    },
    /// Template - Substitute vars in <templatein> and write to <templateout> or STDOUT
    #[structopt(name = "template", visible_alias = "t")]
    Template {
        /// Input Template file
        #[structopt(parse(from_os_str))]
        templatein: PathBuf,
        /// Output Template file, stdout if not present
        #[structopt(parse(from_os_str))]
        templateout: Option<PathBuf>,
    },
    /// Copy Parameter's Value from origin key to destination key
    #[structopt(name = "clone", visible_alias = "c")]
    Clone {
        /// Origin Parameter Name
        origin: String,
        /// Destination Parameter Name
        destination: String,
    },
    /// Recursivelly Copy Parameter's Value Renaming From Origin Prefix to Destination Prefix
    #[structopt(name = "clone-all", visible_alias = "ca")]
    CloneAll {
        /// Origin Prefix Name
        prefixorigin: String,
        /// Destination Prefix Name
        prefixdestination: String,
    },
}
