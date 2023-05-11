use diesel::result::Error as DieselError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("resource not found")]
    NotFound,
    #[error("{0}")]
    DBError(DieselError),
    #[error("unsupported type: {0}")]
    UnsupportedType(String),
    #[error("incorrect arguments")]
    MismatchedArgs,
    #[error("not a function")]
    NotAFunction,
}

impl From<DieselError> for Error {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::NotFound => Error::NotFound,
            _ => Self::DBError(error),
        }
    }
}
