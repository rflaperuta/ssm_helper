#[derive(Serialize, Debug)]
pub struct SSMParameter {
    pub name: Option<String>,
    pub p_type: Option<String>,
    pub value: Option<String>,
    pub version: Option<i64>,
}

#[derive(Debug)]
pub struct SSMParameterRequest {
    pub name: String,
    pub with_decryption: Option<bool>,
}

#[derive(Debug)]
pub struct SSMParametersRequest {
    pub names: Vec<String>,
    pub with_decryption: Option<bool>,
}

#[derive(Debug)]
pub struct SSMParametersByPathRequest {
    pub path: String,
    pub with_decryption: Option<bool>,
    pub recursive: Option<bool>,
}

#[derive(Debug, Default)]
pub struct SSMParametersResult {
    pub parameters: Vec<SSMParameter>,
    pub invalid_parameters: Vec<String>,
}

#[derive(Debug)]
pub struct SSMRequestError {
    pub reason: String,
}
