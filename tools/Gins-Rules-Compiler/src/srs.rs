use anyhow::Result;
use prost::Message;
use std::fs::File;
use std::io::Write;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeadlessRule {
    #[prost(string, repeated, tag="1")]
    pub domain: ::prost::alloc::vec::Vec<String>,
    #[prost(string, repeated, tag="2")]
    pub domain_suffix: ::prost::alloc::vec::Vec<String>,
    #[prost(string, repeated, tag="3")]
    pub domain_keyword: ::prost::alloc::vec::Vec<String>,
    #[prost(string, repeated, tag="4")]
    pub domain_regex: ::prost::alloc::vec::Vec<String>,
    #[prost(string, repeated, tag="6")]
    pub ip_cidr: ::prost::alloc::vec::Vec<String>,
    #[prost(uint32, repeated, tag="28")]
    pub asn: ::prost::alloc::vec::Vec<u32>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlainRuleSet {
    #[prost(message, repeated, tag="1")]
    pub rules: ::prost::alloc::vec::Vec<HeadlessRule>,
}

pub fn encode_srs(
    domain: Vec<String>,
    domain_suffix: Vec<String>,
    domain_keyword: Vec<String>,
    domain_regex: Vec<String>,
    ip_cidr: Vec<String>,
    asn: Vec<u32>,
    out_path: &std::path::Path,
) -> Result<()> {
    let mut file = File::create(out_path)?;
    
    let headless = HeadlessRule {
        domain,
        domain_suffix,
        domain_keyword,
        domain_regex,
        ip_cidr,
        asn,
    };
    
    let ruleset = PlainRuleSet {
        rules: vec![headless],
    };
    
    let mut buf = Vec::new();
    ruleset.encode(&mut buf)?;
    
    // SRS v1 Header
    file.write_all(b"SRS\x01")?;
    file.write_all(&buf)?;
    
    Ok(())
}
