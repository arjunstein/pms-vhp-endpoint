use salvo::oapi::{ToParameters, ToSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, ToParameters)]
#[salvo(parameters(default_parameter_in = Query))]
pub struct PmsQueryParams {
    pub mode: String,
    pub room: Option<String>,
    pub oldroom: Option<String>,
    pub name: Option<String>,
    pub pass: Option<String>,
    pub rsvno: Option<String>,
    pub cidate: Option<String>,
    pub codate: Option<String>,
    pub cotime: Option<String>,
    pub gtype: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PmsResponse {
    pub status: String,
    pub message: String,
}
