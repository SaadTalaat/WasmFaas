use std::error::Error;
use std::fmt::{self, Formatter, Display};
use std::process::Command;

#[derive(Debug)]
pub enum CompileError {
    IOError,
    EncodingError,
    CompileError,
}

pub async fn compile(body: &str) -> Result<Vec<u8>, CompileError> {
    // Read the template
    let template = std::fs::read("boilerplate/src/lib.mrs")?;
    // Build the code
    let code = std::str::from_utf8(&template)?.replace("%", body);
    // Write the code to `lib.rs`
    std::fs::write("boilerplate/src/lib.rs", code.as_bytes())?;
    let output = Command::new("cargo")
        .arg("build")
        .arg("--package=boilerplate")
        .arg("--release")
        .arg("--target=wasm32-unknown-unknown")
        .output()?;

    if !output.status.success() {
        Err(CompileError::CompileError)
    } else {
        // Read binary
        let wasm = std::fs::read("./target/wasm32-unknown-unknown/release/boilerplate.wasm")?;
        Ok(wasm)
    }
}

impl From<std::io::Error> for CompileError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError
    }
}

impl From<std::str::Utf8Error> for CompileError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::EncodingError
    }
}

impl Error for CompileError {}
impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let msg = match self {
            Self::IOError => "IO Error while compiling code",
            Self::EncodingError => "Malformed input",
            Self::CompileError => "Couldn't compile source code",
        };
        write!(f, "{}", msg)
    }
}
