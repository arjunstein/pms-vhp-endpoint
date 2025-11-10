use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct PmsQuery {
    pub mode: String,
    pub room: Option<String>,
    #[allow(dead_code)]
    pub oldroom: Option<String>,
    pub name: Option<String>,
    pub pass: Option<String>,
    pub rsvno: Option<String>,
    pub cidate: Option<String>,
    pub codate: Option<String>,
    pub cotime: Option<String>,
    pub gtype: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PmsResponse {
    pub status: String,
    pub message: String,
}
