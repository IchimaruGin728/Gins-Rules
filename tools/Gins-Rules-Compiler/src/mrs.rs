use anyhow::Result;
use serde::Serialize;
use std::fs::File;
use std::io::Write;

#[derive(Serialize)]
struct MrsPayload<'a> {
    payload: Vec<&'a str>,
}

pub fn encode_mrs(payload: Vec<&str>, out_path: &std::path::Path) -> Result<()> {
    let mut file = File::create(out_path)?;
    let wrapper = MrsPayload { payload };
    let mut buf = Vec::new();
    let mut ser = rmp_serde::Serializer::new(&mut buf).with_struct_map();
    wrapper.serialize(&mut ser)?;
    file.write_all(&buf)?;
    Ok(())
}
