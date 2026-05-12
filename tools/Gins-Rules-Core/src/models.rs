use ecow::EcoString;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    DomainRegex,
    IpCidr,
    IpCidr6,
    IpAsn,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub domain: HashSet<EcoString>,
    pub domain_suffix: HashSet<EcoString>,
    pub domain_keyword: HashSet<EcoString>,
    pub domain_regex: HashSet<EcoString>,
    pub ip_cidr: HashSet<EcoString>,
    pub ip_asn: HashSet<EcoString>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: &RuleSet) {
        self.domain.extend(other.domain.iter().cloned());
        self.domain_suffix.extend(other.domain_suffix.iter().cloned());
        self.domain_keyword.extend(other.domain_keyword.iter().cloned());
        self.domain_regex.extend(other.domain_regex.iter().cloned());
        self.ip_cidr.extend(other.ip_cidr.iter().cloned());
        self.ip_asn.extend(other.ip_asn.iter().cloned());
    }

    pub fn is_empty(&self) -> bool {
        self.domain.is_empty()
            && self.domain_suffix.is_empty()
            && self.domain_keyword.is_empty()
            && self.domain_regex.is_empty()
            && self.ip_cidr.is_empty()
            && self.ip_asn.is_empty()
    }

    pub fn len(&self) -> usize {
        self.domain.len()
            + self.domain_suffix.len()
            + self.domain_keyword.len()
            + self.domain_regex.len()
            + self.ip_cidr.len()
            + self.ip_asn.len()
    }
}
