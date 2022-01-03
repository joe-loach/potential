use spirv_builder::*;

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    build("shaders/compute")?;
    Ok(())
}

fn build(crate_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new(crate_path, "spirv-unknown-spv1.5")
        .print_metadata(MetadataPrintout::None)
        .build()?;
    Ok(())
}
