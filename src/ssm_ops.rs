use rusoto_core::Region;
use rusoto_ssm::{Ssm, SsmClient, /*GetParameterRequest,*/ GetParametersRequest, GetParametersByPathRequest};
use std::fmt;

use ssm_parameters::{SSMParametersRequest, SSMParametersByPathRequest, SSMParameter, SSMParametersResult, SSMRequestError};

//#[derive(Debug)]
pub struct SSMOps {
    region: String,
    ssm_client: SsmClient
}

impl fmt::Debug for SSMOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ region: {} }}", self.region)
    }
}

impl SSMOps {
    pub fn new(region: &str) -> Self {
        SSMOps {
            region: region.to_string(),
            ssm_client: SsmClient::simple(region.parse::<Region>().unwrap_or(Region::default()))
        }
    }

    pub fn get_parameters_by_path(&self, req: &SSMParametersByPathRequest) -> Result<SSMParametersResult, SSMRequestError> {
        let mut input: GetParametersByPathRequest = GetParametersByPathRequest { path: req.path.clone(), recursive: req.recursive, with_decryption: req.with_decryption, ..Default::default() };
        let mut out_parms: Vec<SSMParameter> = Vec::new();
        let mut error: Option<SSMRequestError> = None;
        loop {
            match self.ssm_client.get_parameters_by_path(&input).sync() {
                Ok(output) => {
                    match output.parameters {
                        Some(parameter_list) => {
                            out_parms.extend(parameter_list.into_iter()
                                                .map(|p| SSMParameter {name: p.name, p_type: p.type_, value: p.value, version: p.version})
                                                );
                            match output.next_token {
                                Some(token) => {
                                    input = GetParametersByPathRequest { next_token: Some(token.clone()), ..input };
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                        None => {
                            println!("No parameters found!");
                            break;
                        }
                    }
                }
                Err(err) => {
                    error = Some(SSMRequestError { reason: err.to_string()});
                    break;
                }
            }
        }
        
        if let Some(e) = error {
            return Err(e);
        }
        
        Ok(SSMParametersResult {
                                    parameters: out_parms, 
                                    invalid_parameters: vec![]
                                    })
    }

    pub fn get_parameters(&self, req: &SSMParametersRequest) -> Result<SSMParametersResult, SSMRequestError> {
        let input: GetParametersRequest = GetParametersRequest { names: req.names.clone(), with_decryption: req.with_decryption };
        match self.ssm_client.get_parameters(&input).sync() {
            Ok(output) => {
                let invalid_parameters = output.invalid_parameters.unwrap();
                let valid_parameters = output.parameters.unwrap();
                if invalid_parameters.len() > 0 {
                    let parm_list: String = invalid_parameters.iter()
                                                            .map(|p| format!("{}, ", p))
                                                            .collect();
                    let out_msg: String = format!("Invalid Parameters: {}", parm_list);
                    if valid_parameters.len() == 0 {
                        return Err(SSMRequestError {
                            reason: out_msg
                        })
                    }
                }
                let out_parms: Vec<SSMParameter> = valid_parameters
                                                .into_iter()
                                                .map(|p| SSMParameter {name: p.name, p_type: p.type_, value: p.value, version: p.version})
                                                .collect();
                
                Ok(SSMParametersResult {
                                    parameters: out_parms, 
                                    invalid_parameters: invalid_parameters
                                    })
            },
            Err(err) => {
                Err(SSMRequestError { reason: err.to_string()})
            }
        }
    }

}