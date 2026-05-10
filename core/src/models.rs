use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Rules {
    #[serde(default)]
    pub domain_suffix: Vec<String>,
    #[serde(default)]
    pub domain: Vec<String>,
    #[serde(default)]
    pub domain_keyword: Vec<String>,
    #[serde(default)]
    pub domain_regex: Vec<String>,
    #[serde(default)]
    pub ip_cidr: Vec<String>,
    #[serde(default)]
    pub ip_asn: Vec<String>,
    #[serde(default)]
    pub process_name: Vec<String>,
    #[serde(default)]
    pub user_agent: Vec<String>,
}

impl Rules {
    pub fn count(&self) -> usize {
        self.domain_suffix.len()
            + self.domain.len()
            + self.domain_keyword.len()
            + self.domain_regex.len()
            + self.ip_cidr.len()
            + self.ip_asn.len()
            + self.process_name.len()
            + self.user_agent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

pub struct MihomoRuleMode {
    pub behavior: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingBoxRuleSet {
    pub version: i32,
    pub rules: Vec<SingBoxRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingBoxRule {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub domain_suffix: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub domain: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub domain_keyword: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub domain_regex: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ip_cidr: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub process_name: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub user_agent: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnywhereRule {
    #[serde(rename = "type")]
    pub rule_type: i32,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AsnPrefixRecord {
    pub asn: u32,
    pub cidr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Cidr {
    #[prost(bytes, tag = "1")]
    pub ip: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "2")]
    pub prefix: u32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoIp {
    #[prost(string, tag = "1")]
    pub country_code: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub cidr: ::prost::alloc::vec::Vec<Cidr>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoIpList {
    #[prost(message, repeated, tag = "1")]
    pub entry: ::prost::alloc::vec::Vec<GeoIp>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DomainType {
    Plain = 0,
    Regex = 1,
    Domain = 2,
    Full = 3,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Domain {
    #[prost(enumeration = "DomainType", tag = "1")]
    pub r#type: i32,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoSite {
    #[prost(string, tag = "1")]
    pub country_code: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub domain: ::prost::alloc::vec::Vec<Domain>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoSiteList {
    #[prost(message, repeated, tag = "1")]
    pub entry: ::prost::alloc::vec::Vec<GeoSite>,
}

#[derive(Serialize)]
pub struct CountryRecord {
    pub country: CountryIso,
}

#[derive(Serialize)]
pub struct CountryIso {
    #[serde(rename = "iso_code")]
    pub iso_code: String,
}

#[derive(Serialize)]
pub struct AsnRecord {
    pub autonomous_system_number: u32,
    pub autonomous_system_organization: String,
}
