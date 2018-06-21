#[macro_use]
extern crate structopt;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate rusoto_core;
extern crate rusoto_ssm;

use std::process;

use structopt::StructOpt;

use args::*;
use ssm_ops::*;
use ssm_parameters::*;

mod args;
mod ssm_ops;
mod ssm_parameters;

/// AWS SSM Helper
/// Command Line
/// --region, -r => Set region for search
/// --decrypt, -d => Decrypt parameter value
/// --quiet => No unnecessary output
/// list-all, lp, all => Lists all parameters
/// get, g PARAM => get param by name(path)
/// template, t, FILENAME_IN.tpl [FILENAME_OUT.ext] => parse template and substitute named paths
/// clone <origin> <destination>, c <origin> <destination> => Copy a Parameter's Value from origin key to destination key
/// 
/// TODO
/// Implement:
/// - Quiet
/// - Template
/// - Clone
/// - Fail Crate
/// - Default for Requests?
/// Improve:
/// - pagination calls
fn main() {
    let clap_options = Opt::clap().get_matches_safe();

    // Will exit with error code 1 even for VersionDisplayed and HelpDisplayed
    if let Err(err) = clap_options {
        println!("{}", err.message);
        process::exit(1)
    }

    let options = Opt::from_clap(&clap_options.unwrap());

    let decrypt = options.decrypt;
    let ssm = SSMOps::new(&options.region);

    match options.cmd {
        Command::Get{ name } => {
            ssm.get_parameters(&SSMParametersRequest { names: name, with_decryption: Some(decrypt) }).unwrap()
                    .parameters.into_iter()
                    .for_each(|p| println!("{}", serde_json::to_string(&p).unwrap()));
        },
        Command::ListAll{} => {
            ssm.get_parameters_by_path(&SSMParametersByPathRequest { path: String::from("/"), recursive: Some(true), with_decryption: Some(decrypt) }).unwrap()
                    .parameters.into_iter()
                    .for_each(|p| println!("{}", serde_json::to_string(&p).unwrap()));
        },
    //     Command::Template{ templatein, templateout } => println!("IN: {:#?} - OUT: {:#?}", templatein, templateout),
    //     Command::Clone{ origin, destination } => println!("Origin: {:#?} - Destination: {:#?}", origin, destination)
        _ => {()}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1, 1); // ;-)
    }

    #[test]
    fn get_parameter() {
        let decrypt = true;
        let ssm = SSMOps::new("us-east-1");
        let name = vec!["/test/ssm_helper/param1".to_string()];
        let result = ssm.get_parameters(&SSMParametersRequest { names: name, with_decryption: Some(decrypt) });
        assert!(result.is_ok());
        let unw_result = result.unwrap();
        assert!(unw_result.parameters.len() > 0);
        assert!(unw_result.invalid_parameters.len() == 0);
    }

    #[test]
    fn get_parameter_and_error() {
        let decrypt = true;
        let ssm = SSMOps::new("us-east-1");
        let name = vec!["/test/ssm_helper/one".to_string(), "/dev/asdasdasd".to_string()];
        let result = ssm.get_parameters(&SSMParametersRequest { names: name, with_decryption: Some(decrypt) });
        assert!(result.is_ok());
        let unw_result = result.unwrap();
        assert!(unw_result.parameters.len() == 1);
        assert!(unw_result.invalid_parameters.len() == 1);
    }

    #[test]
    fn get_parameter_error() {
        let decrypt = true;
        let ssm = SSMOps::new("us-east-1");
        let name = vec!["/asdasdasd".to_string()];
        let result = ssm.get_parameters(&SSMParametersRequest { names: name, with_decryption: Some(decrypt) });
        assert!(result.is_err());
    }

    #[test]
    fn get_parameters_by_path() {
        let decrypt = true;
        let ssm = SSMOps::new("us-east-1");
        let path = "/".to_string();
        let recursive = true;
        let result = ssm.get_parameters_by_path(&SSMParametersByPathRequest { path: path, with_decryption: Some(decrypt), recursive: Some(recursive) });
        assert!(result.is_ok());
        let unw_result = result.unwrap();
        assert!(unw_result.parameters.len() > 0);
        assert!(unw_result.invalid_parameters.len() == 0);
    }

    #[test]
    fn get_parameters_by_path_error() {
        let decrypt = true;
        let ssm = SSMOps::new("us-east-1");
        let path = "*".to_string();
        let recursive = true;
        let result = ssm.get_parameters_by_path(&SSMParametersByPathRequest { path: path, with_decryption: Some(decrypt), recursive: Some(recursive) });
        assert!(result.is_err());
    }
}