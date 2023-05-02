
use std::process::Command;
use std::error::Error;

pub async fn compile(body: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    // Read the template
    let template = std::fs::read("boilerplate/src/lib.mrs")?;
    // Build the code
    let code = std::str::from_utf8(&template)?.replace("%", body);
    // Write the code to `lib.rs`
    std::fs::write("boilerplate/src/lib.rs", code.as_bytes())?;
    Command::new("cargo")
        .arg("build")
        .arg("--package=boilerplate")
        .arg("--release")
        .arg("--target=wasm32-unknown-unknown")
        .output()?;
    // Read binary
    let wasm = std::fs::read("./target/wasm32-unknown-unknown/release/boilerplate.wasm")?;
    Ok(wasm)
}
