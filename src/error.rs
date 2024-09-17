use std::{error::Error, fmt, fmt::Display};

#[derive(Clone, Debug)]
pub enum ObsEnvError {
    ERROR(String),
    GIT(String),
}

impl Error for ObsEnvError {}

impl Display for ObsEnvError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self {
            ObsEnvError::ERROR(err_msg) => write!(f, "ERROR: {}", err_msg),
            ObsEnvError::GIT(err_msg) => write!(f, "GIT: {}", err_msg),
        }
    }
}
