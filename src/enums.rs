#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidIndex,
    InvalidTimeframe,
    FetchError,
    ParseError,
    TooManyRequests,
}
