use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use spirv_builder::*;

const SHADERS: &[&'static str] = &["compute"];

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    shaders: HashMap<String, ShaderInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ShaderInfo {
    entries: Vec<String>,
    module: PathBuf,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let built = SHADERS
        .iter()
        .map(|shader| match build(shader) {
            Ok(built) => Ok((shader, built)),
            Err(e) => Err(e),
        })
        .map(|res| {
            res.map(|(name, info)| {
                (
                    name,
                    ShaderInfo {
                        entries: info.entry_points,
                        module: info.module.unwrap_single().to_owned(),
                    },
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let config = built
        .into_iter()
        .fold(Config::default(), |mut acc, (name, info)| {
            acc.shaders.insert(name.to_string(), info);
            acc
        });

    let toml = toml::to_string(&config)?;

    let mut shader_toml = File::options()
        .read(true)
        .create(true)
        .write(true)
        .truncate(true)
        .open("shaders.toml")?;

    write!(shader_toml, "{}", toml)?;

    Ok(())
}

fn build(crate_name: impl Into<PathBuf>) -> Result<CompileResult, Box<dyn std::error::Error>> {
    let mut path: PathBuf = "shaders".into();
    let crate_name: PathBuf = crate_name.into();
    path.push(crate_name);
    Ok(SpirvBuilder::new(path, "spirv-unknown-spv1.5")
        .print_metadata(MetadataPrintout::None)
        .build()?)
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("BUILD FAILED {:?}", e);
            std::process::exit(1);
        }
    }
}
