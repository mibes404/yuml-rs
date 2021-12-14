use derive_more::{Display, Error, From};

#[derive(Default, Debug, Display, Error)]
#[display(fmt = "options error: {}", message)]
pub struct OptionsError {
    message: String,
}

impl OptionsError {
    pub fn new(message: &str) -> Self {
        OptionsError {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Display, Error, From)]
pub enum YumlError {
    Options {
        source: OptionsError,
    },
    #[display(fmt = "Invalid Expression")]
    Expression,
    Format {
        source: std::fmt::Error,
    },
    Io {
        source: std::io::Error,
    },
    #[display(fmt = "Invalid yUML file: {}", _.0)]
    InvalidFile(#[error(not(source))] String),
}

pub type YumlResult<T> = Result<T, YumlError>;
