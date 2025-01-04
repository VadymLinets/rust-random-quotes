#[derive(Debug)]
pub enum Error {
    ErrNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ErrNotFound => write!(f, "Database record not found"),
        }
    }
}
