use std::fmt;
use std::path::PathBuf;
use std::io::Write;

use rusoto_core::Region;
use rusoto_ssm::{Ssm, SsmClient, GetParametersRequest, GetParametersByPathRequest};

use ssm_parameters::{SSMParametersRequest, SSMParametersByPathRequest, SSMParameter, SSMParametersResult, SSMRequestError};

use handlebars::{Handlebars, RenderContext, Helper, JsonRender, HelperResult, Context, Output, template};

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

    pub fn process_template(&self, template_in: PathBuf, template_out: Option<PathBuf>) {
        if !template_in.is_file() {
            println!("Not a Valid File: {}", template_in.to_str().unwrap()); // ???
            return (); // Return early as a Error, must be a valid file
        }

        // Else
        // let template = ::liquid::ParserBuilder::with_liquid()
        //     .extra_filters()
        //     .filter("ssm", ssm_template::ssm_filter as interpreter::FnFilterValue)
        //     .tag("ssm", ssm_template::ssm_tag as compiler::FnParseTag)
        //     .build()
        //     .parse_file(template_in.as_path())
        //     .unwrap();

        // let globals = ::liquid::Object::new();
        // // globals.insert("num".to_owned(), ::liquid::Value::scalar(4f32));

        // let output = template.render(&globals).unwrap();
        // println!("OUTPUT: {:#?}", output);

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper("ssm",
            Box::new(|h: &Helper, r: &Handlebars, _: &Context, rc: &mut RenderContext, out: &mut Output| -> HelperResult {
                let param = h.param(0).unwrap();
                
                out.write("ssm helper: ")?;
                out.write(param.value().render().as_ref())?;
                Ok(())
            }));

        match handlebars.register_template_file("template_in", &template_in.as_path()) {
            Err(error) => {
                println!("TEMPLATE ERROR: {:#?}", error);
            },
            _ => ()
        }

        match handlebars.get_template("template_in") {
            Some(template) => {
                println!("TEMPLATE: {:#?}", template);
                for element in template.elements.iter() {
                    println!("ELEMENT: {:#?}", element);
                    match element {
                        template::TemplateElement::HelperExpression(he) => {
                            println!("EXPRESSION: {:#?}", he);
                            println!("PARAMS: {:#?}", he.params);
                        }
                        _ => ()
                    }
                }
            },
            None => {
                println!("Template not found!");
            }
        }

        match handlebars.render("template_in", &()) {
            Ok(output) => {
                println!("OK.: {:#?}", output);
            },
            Err(e) => {
                println!("ERR: {:#?}", e);
            }
        }
    }

}