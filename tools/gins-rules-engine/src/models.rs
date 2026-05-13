use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub domain: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub domain_suffix: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub domain_keyword: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub domain_regex: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub domain_wildcard: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub ip_cidr: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub ip_asn: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub process_name: HashSet<CompactString>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub user_agent: HashSet<CompactString>,
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
        self.domain_wildcard.extend(other.domain_wildcard.iter().cloned());
        self.ip_cidr.extend(other.ip_cidr.iter().cloned());
        self.ip_asn.extend(other.ip_asn.iter().cloned());
        self.process_name.extend(other.process_name.iter().cloned());
        self.user_agent.extend(other.user_agent.iter().cloned());
    }

    pub fn is_empty(&self) -> bool {
        self.domain.is_empty()
            && self.domain_suffix.is_empty()
            && self.domain_keyword.is_empty()
            && self.domain_regex.is_empty()
            && self.domain_wildcard.is_empty()
            && self.ip_cidr.is_empty()
            && self.ip_asn.is_empty()
            && self.process_name.is_empty()
            && self.user_agent.is_empty()
    }

    pub fn len(&self) -> usize {
        self.domain.len()
            + self.domain_suffix.len()
            + self.domain_keyword.len()
            + self.domain_regex.len()
            + self.domain_wildcard.len()
            + self.ip_cidr.len()
            + self.ip_asn.len()
            + self.process_name.len()
            + self.user_agent.len()
    }

    pub fn has_complex_types(&self) -> bool {
        !self.domain_keyword.is_empty()
            || !self.domain_regex.is_empty()
            || !self.domain_wildcard.is_empty()
            || !self.ip_cidr.is_empty()
            || !self.ip_asn.is_empty()
            || !self.process_name.is_empty()
            || !self.user_agent.is_empty()
    }
}
