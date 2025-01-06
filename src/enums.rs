use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidIndex,
    InvalidTimeframe,
    FetchError(String),
    ParseError(String),
    TooManyRequests,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidAddress => todo!(),
            Error::InvalidIndex => todo!(),
            Error::InvalidTimeframe => todo!(),
            Error::FetchError(msg) => write!(f, "Fetch error: {}", msg),
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::TooManyRequests => todo!(),
        }
    }
}
