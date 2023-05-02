use serde::Serialize;

#[derive(Serialize)]
enum StatusKind {
    Success,
    Failure,
}

#[derive(Serialize)]
enum ErrorKind {
    BadRequest,
}

#[derive(Serialize)]
pub struct Status {
    result: StatusKind,
    message: Option<ErrorKind>,
}

impl Status {
    pub fn ok() -> Self {
        Self {
            result: StatusKind::Success,
            message: None,
        }
    }
}
