use serde_json::value::Value as Json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fmt, fs};

use rusoto_core::{Region, RusotoError};
use rusoto_ssm::{
    GetParameterError, GetParameterRequest, GetParametersByPathRequest, GetParametersRequest,
    PutParameterError, PutParameterRequest, Ssm, SsmClient,
};

use crate::ssm_parameters::{
    SSMParameter, SSMParameterRequest, SSMParametersByPathRequest, SSMParametersRequest,
    SSMParametersResult, SSMRequestError,
};

use failure::Error;
use handlebars::{
    template, Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext,
    RenderError,
};

//#[derive(Debug)]
pub struct SSMOps {
    region: String,
    ssm_client: SsmClient,
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
            ssm_client: SsmClient::new(region.parse::<Region>().unwrap_or_default()),
        }
    }

    pub fn get_parameters_by_path(
        &self,
        req: &SSMParametersByPathRequest,
    ) -> Result<SSMParametersResult, SSMRequestError> {
        let mut input: GetParametersByPathRequest = GetParametersByPathRequest {
            path: req.path.clone(),
            recursive: req.recursive,
            with_decryption: req.with_decryption,
            ..Default::default()
        };
        let mut out_parms: Vec<SSMParameter> = Vec::new();
        let mut error: Option<SSMRequestError> = None;
        loop {
            match self.ssm_client.get_parameters_by_path(input.clone()).sync() {
                Ok(output) => match output.parameters {
                    Some(parameter_list) => {
                        out_parms.extend(parameter_list.into_iter().map(|p| SSMParameter {
                            name: p.name,
                            p_type: p.type_,
                            value: p.value,
                            version: p.version,
                        }));
                        match output.next_token {
                            Some(token) => {
                                input = GetParametersByPathRequest {
                                    next_token: Some(token.clone()),
                                    ..input
                                };
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
                },
                Err(err) => {
                    error = Some(SSMRequestError {
                        reason: err.to_string(),
                    });
                    break;
                }
            }
        }

        if let Some(e) = error {
            return Err(e);
        }

        Ok(SSMParametersResult {
            parameters: out_parms,
            invalid_parameters: vec![],
        })
    }

    pub fn get_parameters(&self, req: &SSMParametersRequest) -> Result<SSMParametersResult, Error> {
        let input: GetParametersRequest = GetParametersRequest {
            names: req.names.clone(),
            with_decryption: req.with_decryption,
        };
        match self.ssm_client.get_parameters(input).sync() {
            Ok(output) => {
                let invalid_parameters = output.invalid_parameters.unwrap();
                let valid_parameters = output.parameters.unwrap();
                if !invalid_parameters.is_empty() {
                    let parm_list: String = invalid_parameters
                        .iter()
                        .map(|p| format!("{}, ", p))
                        .collect();
                    let out_msg: String = format!("Invalid Parameters: {}", parm_list);
                    if valid_parameters.is_empty() {
                        return Err(failure::err_msg(out_msg));
                    }
                }
                let out_parms: Vec<SSMParameter> = valid_parameters
                    .into_iter()
                    .map(|p| SSMParameter {
                        name: p.name,
                        p_type: p.type_,
                        value: p.value,
                        version: p.version,
                    })
                    .collect();

                Ok(SSMParametersResult {
                    parameters: out_parms,
                    invalid_parameters,
                })
            }
            Err(err) => Err(failure::err_msg(err.to_string())),
        }
    }

    pub fn process_template(
        &self,
        template_in: PathBuf,
        template_out: Option<PathBuf>,
    ) -> Result<(), Error> {
        if !template_in.is_file() {
            return Err(failure::err_msg(format!(
                "Not a Valid File: {}",
                template_in.to_str().unwrap()
            ))); // Return early as an Error, must be a valid file
        }

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper(
            "ssm",
            Box::new(
                |h: &Helper,
                 _r: &Handlebars,
                 ctx: &Context,
                 _rc: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let param = h
                        .param(0)
                        .ok_or_else(|| RenderError::new("SSM Parameter name Required."))?;

                    //                    println!("CTX: {:#?}", ctx);
                    //                    println!("PARAM: {:#?}", param);

                    let null = Json::Null;
                    let value = match ctx.data().as_object() {
                        None => &null,
                        Some(ref o) => o.get(param.value().as_str().unwrap()).unwrap(),
                    };

                    //                    let rendered = format!("{}->{}", param.value().render(), value.render());
                    let rendered = format!("{}", value.render());
                    out.write(rendered.as_ref())?;

                    Ok(())
                },
            ),
        );

        if let Err(error) = handlebars.register_template_file("template", &template_in.as_path()) {
            //            println!("TEMPLATE ERROR: {:#?}", error);
            bail!(error);
        }

        let parameter_list: Vec<String> =
            self.extract_parameters(handlebars.get_template("template"))?;

        //        for p in &parameter_list {
        //            println!("PARAMETER: {:#?}", p);
        //        }

        let parameter_list = self.retrieve_parameters(parameter_list)?;

        match handlebars.render("template", &parameter_list) {
            Err(e) => {
                bail!(e);
            }
            Ok(template_rendered) => match template_out {
                Some(out_file) => {
                    fs::write(out_file, template_rendered)?;
                }
                None => {
                    println!("{}", template_rendered);
                }
            },
        }

        Ok(())
    }

    pub fn clone_parameter(
        &self,
        origin: String,
        destination: String,
        overwrite: bool,
    ) -> Result<(), Error> {
        println!("Origin: {:#?} - Destination: {:#?}", origin, destination);

        let source: SSMParameterRequest = SSMParameterRequest {
            name: origin.clone(),
            with_decryption: Some(false),
        };

        let source_param = self.get_one(source)?;

        let dest: SSMParameter = SSMParameter {
            name: Some(destination.clone()),
            p_type: source_param.p_type,
            value: source_param.value,
            version: None,
        };

        self.put_one(dest, overwrite)?;

        Ok(())
    }

    pub fn clone_recursive(
        &self,
        origin: String,
        destination: String,
        overwrite: bool,
    ) -> Result<(), Error> {
        println!("Origin: {:#?} - Destination: {:#?} - Overwrite: {:#?}", origin, destination, overwrite);

//        let source: SSMParameterRequest = SSMParameterRequest {
//            name: origin.clone(),
//            with_decryption: Some(false),
//        };
//
//        let source_param = self.get_one(source)?;
//
//        let dest: SSMParameter = SSMParameter {
//            name: Some(destination.clone()),
//            p_type: source_param.p_type,
//            value: source_param.value,
//            version: None,
//        };
//
//        self.put_one(dest, overwrite)?;

        Ok(())
    }

    fn get_one(&self, parameter: SSMParameterRequest) -> Result<SSMParameter, Error> {
        let input = GetParameterRequest {
            name: parameter.name.clone(),
            with_decryption: parameter.with_decryption,
        };

        match self.ssm_client.get_parameter(input.clone()).sync() {
            Err(err) => match err {
                RusotoError::Service(s_err) => match s_err as GetParameterError {
                    GetParameterError::InternalServerError(_) => {
                        Err(failure::err_msg("An error occurred on the server side."))
                    }
                    GetParameterError::InvalidKeyId(_) => {
                        Err(failure::err_msg("The query key ID is not valid."))
                    }
                    GetParameterError::ParameterNotFound(_) => Err(
                        format_err!("The parameter \'{}\' could not be found. Verify the name and try again.", parameter.name),
                    ),
                    GetParameterError::ParameterVersionNotFound(_) => {
                        Err(failure::err_msg("The specified parameter version was not found. Verify the parameter name and version, and try again."))
                    }
                },
                RusotoError::HttpDispatch(h_err) => {
                    println!("{:?}", h_err);
                    Err(failure::err_msg(h_err.to_string()))
                }
                RusotoError::Credentials(c_err) => {
                    println!("{:?}", c_err);
                    Err(failure::err_msg(c_err.to_string()))
                }
                RusotoError::Validation(v_err) => {
                    println!("{:?}", v_err);
                    Err(failure::err_msg(v_err.to_string()))
                }
                RusotoError::ParseError(p_err) => {
                    println!("{:?}", p_err);
                    Err(failure::err_msg(p_err.to_string()))
                }
                RusotoError::Unknown(_) => Err(failure::err_msg("Unknown Error.")),
            },
            Ok(res) => {
                println!("{:?}", res);
                let parm = res.parameter.unwrap();
                Ok(SSMParameter{name: parm.name, value: parm.value, p_type: parm.type_, version: parm.version})
            }
        }
    }

    ///
    /// GetParameter
    /// {
    //   "Parameter": {
    //      "ARN": "string",
    //      "LastModifiedDate": number,
    //      "Name": "string",
    //      "Selector": "string",
    //      "SourceResult": "string",
    //      "Type": "string",
    //      "Value": "string",
    //      "Version": number
    //   }
    // }
    /// PutParameter
    /// {
    //   "Name": "string",
    //   "Overwrite": boolean,
    //   "Tier": "string",
    //   "Type": "string",
    //   "Value": "string"
    /// }
    ///    

    fn put_one(&self, parameter: SSMParameter, overwrite: bool) -> Result<(), Error> {
        let input: PutParameterRequest = PutParameterRequest {
            allowed_pattern: None,
            description: None,
            key_id: None,
            name: parameter
                .name
                .expect("Put Parameter: Invalid Parameter Name in Request."),
            overwrite: Some(overwrite),
            policies: None,
            tags: None,
            tier: Some("Standard".to_string()),
            type_: parameter
                .p_type
                .expect("Put Parameter: Invalid Parameter Type in Request."),
            value: parameter
                .value
                .expect("Put Parameter: Invalid Parameter Value in Request."),
        };

        match self.ssm_client.put_parameter(input.clone()).sync() {
            Err(err) => match err {
                RusotoError::Service(s_err) => match s_err as PutParameterError {
                    PutParameterError::InternalServerError(_) => {
                        Err(failure::err_msg("An error occurred on the server side."))
                    }
                    PutParameterError::InvalidKeyId(_) => {
                        Err(failure::err_msg("The query key ID is not valid."))
                    }
                    PutParameterError::HierarchyLevelLimitExceeded(_) => {
                        Err(failure::err_msg("A hierarchy can have a maximum of 15 levels."))
                    }
                    PutParameterError::HierarchyTypeMismatch(_) => {
                        Err(failure::err_msg("Parameter Store does not support changing a parameter type in a hierarchy. For example, you can't change a parameter from a String type to a SecureString type. You must create a new, unique parameter."))
                    }
                    PutParameterError::IncompatiblePolicy(_) => {
                        Err(failure::err_msg("There is a conflict in the policies specified for this parameter."))
                    }
                    PutParameterError::InvalidAllowedPattern(_) => {
                        Err(failure::err_msg("The request does not meet the regular expression requirement."))
                    }
                    PutParameterError::InvalidPolicyAttribute(_) => {
                        Err(failure::err_msg("A policy attribute or its value is invalid."))
                    }
                    PutParameterError::InvalidPolicyType(_) => {
                        Err(failure::err_msg("he policy type is not supported. Parameter Store supports the following policy types: Expiration, ExpirationNotification, and NoChangeNotification."))
                    }
                    PutParameterError::ParameterAlreadyExists(_) => Err(
                        format_err!("The parameter \'{}\' already exists. You can't create duplicate parameters. Set --overwrite if you want change the same parameter.", input.name.clone()),
                    ),
                    PutParameterError::ParameterLimitExceeded(_) => {
                        Err(failure::err_msg("You have exceeded the number of parameters for this AWS account. Delete one or more parameters and try again."))
                    }
                    PutParameterError::ParameterMaxVersionLimitExceeded(_) => {
                        Err(failure::err_msg("The parameter exceeded the maximum number of allowed versions."))
                    }
                    PutParameterError::ParameterPatternMismatch(_) => {
                        Err(failure::err_msg("The parameter name is not valid."))
                    }
                    PutParameterError::PoliciesLimitExceeded(_) => {
                        Err(failure::err_msg("You specified more than the maximum number of allowed policies for the parameter. The maximum is 10."))
                    }
                    PutParameterError::TooManyUpdates(_) => {
                        Err(failure::err_msg("here are concurrent updates for a resource that supports one update at a time."))
                    }
                    PutParameterError::UnsupportedParameterType(_) => Err(
                        format_err!("The parameter type \'{}\' is not supported.", input.type_.clone()),
                    ),
                },
                RusotoError::HttpDispatch(h_err) => {
                    println!("{:?}", h_err);
                    Err(failure::err_msg(h_err.to_string()))
                }
                RusotoError::Credentials(c_err) => {
                    println!("{:?}", c_err);
                    Err(failure::err_msg(c_err.to_string()))
                }
                RusotoError::Validation(v_err) => {
                    println!("{:?}", v_err);
                    Err(failure::err_msg(v_err.to_string()))
                }
                RusotoError::ParseError(p_err) => {
                    println!("{:?}", p_err);
                    Err(failure::err_msg(p_err.to_string()))
                }
                RusotoError::Unknown(_) => Err(failure::err_msg("Unknown Error.")),
            },
            Ok(res) => {
                println!("{:?}", res);
                //                let parm = res.parameter.unwrap();
                //                Ok(SSMParameter{name: parm.name, value: parm.value, p_type: parm.type_, version: parm.version})
                Ok(())
            }
        }
    }

    fn extract_parameters(
        &self,
        template: Option<&handlebars::template::Template>,
    ) -> Result<Vec<String>, Error> {
        let result: Vec<String> = template
            .ok_or(format_err!("Template Unavailable"))?
            .elements
            .iter()
            .filter_map(|element| match element {
                template::TemplateElement::Expression(he)
                    if he.name.as_name().unwrap() == "ssm".to_owned() && he.params.len() == 1 =>
                {
                    match &he.params[0] {
                        handlebars::template::Parameter::Literal(a) => a.as_str().map(String::from),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();

        Ok(result)
    }

    fn retrieve_parameters(
        &self,
        parameters: Vec<String>,
    ) -> Result<HashMap<String, String>, Error> {
        let gp = SSMParametersRequest {
            with_decryption: Some(true),
            names: parameters,
        };

        match self.get_parameters(&gp) {
            Ok(result) => {
                if result.invalid_parameters.len() > 0 {
                    return Err(failure::err_msg(format_err!(
                        "Invalid Parameters: {}",
                        result.invalid_parameters.join(", "),
                    )));
                } else {
                    let mut data: HashMap<String, String> = HashMap::new();
                    result.parameters.iter().for_each(|p| {
                        data.insert(p.name.clone().unwrap(), p.value.clone().unwrap());
                    });
                    Ok(data.clone())
                }
            }
            Err(e) => Err(e),
        }
    }
}
