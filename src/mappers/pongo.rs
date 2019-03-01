use crate::checks::domain::*;
use crate::checks::page::*;

//
// Data structures based on private Centra API, called "Pongo":
//

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHost {

    /// Domains to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Domains>,

    /// Pages to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Pages>,

    /// Updated at:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// Client name:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,

    /// Client is active?:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Client data:
    pub data: PongoHostData,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHostData {

    /// Client application environment:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,

    /// Client application ams name:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ams: Option<String>,

    /// Client main host name:
    pub host: PongoHostDetails,

    /// Client report:
    pub report: PongoReport,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoReport {

    /// Application modules enabled:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<String>>,

    /// Application processes:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processes: Option<String>,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHostDetails {

    /// Host IPv4:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    /// Primary host name:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_vhost: Option<String>,

    /// List of virtual hosts of client:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vhosts: Option<Vec<String>>,

    /// Backend SSHD port of client:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_port: Option<String>,

    /// Showroom urls of client:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub showroom_urls: Option<Vec<String>>,

}


/// PongoHosts collection type
pub type PongoHosts = Vec<PongoHost>;


