use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    SingBox,
    Srs,
    Mihomo,
    Mrs,
    Stash,
    Surge,
    Shadowrocket,
    Loon,
    QuantumultX,
    Surfboard,
    Exclave,
    Anywhere,
    Egern,
    Text,
}

impl Format {
    pub fn all() -> &'static [Format] {
        &[
            Format::SingBox,
            Format::Srs,
            Format::Mihomo,
            Format::Mrs,
            Format::Stash,
            Format::Surge,
            Format::Shadowrocket,
            Format::Loon,
            Format::QuantumultX,
            Format::Surfboard,
            Format::Exclave,
            Format::Anywhere,
            Format::Egern,
            Format::Text,
        ]
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "singbox" => Ok(Format::SingBox),
            "srs" => Ok(Format::Srs),
            "mihomo" => Ok(Format::Mihomo),
            "mrs" => Ok(Format::Mrs),
            "stash" => Ok(Format::Stash),
            "surge" => Ok(Format::Surge),
            "shadowrocket" => Ok(Format::Shadowrocket),
            "loon" => Ok(Format::Loon),
            "quantumultx" => Ok(Format::QuantumultX),
            "surfboard" => Ok(Format::Surfboard),
            "exclave" => Ok(Format::Exclave),
            "anywhere" => Ok(Format::Anywhere),
            "egern" => Ok(Format::Egern),
            "text" => Ok(Format::Text),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}
