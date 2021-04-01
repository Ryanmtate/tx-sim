use csv::{Error as CsvError, IntoInnerError, Writer};
use std::num::ParseFloatError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("I/O Error")]
    IoError(#[from] std::io::Error),
    #[error("CSV Error")]
    CsvError(#[from] CsvError),
    #[error("CSV Writer Error")]
    CsvWriterError(#[from] IntoInnerError<Writer<Vec<u8>>>),
    #[error("Failed to parse amount")]
    ParseFloatError(#[from] ParseFloatError),
}
