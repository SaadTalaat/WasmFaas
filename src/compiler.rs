use std::path::Path;
use std::process::Command;
use thiserror::Error;

pub struct Compiler {
    dir: String,
}

impl Compiler {
    pub fn new(dir: &str) -> Self {
        Self {
            dir: dir.to_owned(),
        }
    }

    pub async fn compile(&self, body: &str) -> Result<Vec<u8>, CompileError> {
        let path_str = format!("{}/src/lib.mrs", self.dir);
        let path = Path::new(&path_str);
        tracing::trace!(
            "Reading template code: {:?}, {:?}",
            path,
            tokio::fs::canonicalize(&path).await
        );
        // Read the template
        let template = tokio::fs::read(path).await?;
        tracing::trace!("Constructing source file from template");
        // Build the code
        let code = std::str::from_utf8(&template)?.replace("%", body);
        tracing::trace!("Writing source code to lib.rs");
        // Write the code to `lib.rs`
        let out_path_str = format!("{}/src/lib.rs", self.dir);
        let out_path = Path::new(&out_path_str);
        tokio::fs::write(out_path, code.as_bytes()).await?;
        // Compile the code
        tracing::trace!("Compiling code to wasm binary");
        let output = Command::new("cargo")
            .arg("build")
            .arg("--package=boilerplate")
            .arg("--release")
            .arg("--target=wasm32-unknown-unknown")
            .output()?;

        if !output.status.success() {
            tracing::trace!("Compilation failed");
            Err(CompileError::Generic)
        } else {
            // Read binary
            let wasm =
                tokio::fs::read("./target/wasm32-unknown-unknown/release/boilerplate.wasm").await?;
            Ok(wasm)
        }
    }
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("CompileError::IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("CompileError::EncodingError: {0}")]
    EncodingError(#[from] std::str::Utf8Error),
    #[error("CompileError: failed to compile code")]
    Generic,
}
