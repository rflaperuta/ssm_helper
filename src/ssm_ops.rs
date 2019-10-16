use serde_json::value::Value as Json;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use rusoto_core::Region;
use rusoto_ssm::{GetParametersByPathRequest, GetParametersRequest, Ssm, SsmClient};

use crate::ssm_parameters::{
    SSMParameter, SSMParametersByPathRequest, SSMParametersRequest, SSMParametersResult,
    SSMRequestError,
};

use handlebars::{
    template, Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext,
    RenderError,
};
//use handlebars::template::HelperTemplate;

//use serde_json::value::Value as Json;
use failure::Error;

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
            //; // ???
            return Err(failure::err_msg(format!(
                "Not a Valid File: {}",
                template_in.to_str().unwrap()
            ))); // Return early as a Error, must be a valid file
        }

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper(
            "ssm",
            Box::new(
                |h: &Helper,
                 r: &Handlebars,
                 ctx: &Context,
                 rc: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let param = h
                        .param(0)
                        .ok_or_else(|| RenderError::new("SSM Parameter name Required."))?;

                    println!("CTX: {:#?}", ctx);
                    println!("PARAM: {:#?}", param);

                    let null = Json::Null;
                    let value = match ctx.data().as_object() {
                        None => &null,
                        Some(ref o) => o.get(param.value().as_str().unwrap()).unwrap(),
                    };

                    let rendered = format!("{}->{}", param.value().render(), value.render());
                    out.write(rendered.as_ref())?;

                    Ok(())
                },
            ),
        );

        if let Err(error) = handlebars.register_template_file("template_in", &template_in.as_path())
        {
            println!("TEMPLATE ERROR: {:#?}", error);
        }

        let parameter_list: Vec<String> = self
            .extract_parameters(handlebars.get_template("template_in"))
            .unwrap();

        for p in &parameter_list {
            println!("PARAMETER: {:#?}", p);
        }

        let parameter_list = self.retrieve_parameters(parameter_list)?;

        match handlebars.render("template_in", &parameter_list) {
            Ok(output) => {
                println!("OK.: {:#?}", output);
            }
            Err(e) => {
                println!("ERR: {:#?}", e);
            }
        }

        Ok(())
    }

    fn extract_parameters(
        &self,
        template: Option<&handlebars::template::Template>,
    ) -> Result<Vec<String>, ()> {
        let result: Vec<String> = template
            .unwrap()
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
                    Err(failure::err_msg(result.invalid_parameters.join(", ")))
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

        //        Ok(vec![SSMParametersResult{..Default::default()}])
    }
}
