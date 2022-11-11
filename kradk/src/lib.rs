pub mod krad;
pub mod radk;

/// `Result` wrapper for `Error`
pub type Result<T> = std::result::Result<T, Error>;

/// Potential `Error`s
#[derive(Debug, PartialEq)]
pub enum Error {
    IO,
    Decode,
    Parse(NomError<String>),
}

// convenience def
type NomError<T> = nom::Err<nom::error::Error<T>>;

/// Decode the raw file into UTF-8
pub fn decode(input: &[u8]) -> Result<String> {
    encoding_rs::EUC_JP
        .decode_without_bom_handling_and_without_replacement(input)
        .map(|s| s.into_owned())
        .ok_or(Error::Decode)
}

/// Read and decode a given file
pub fn read<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
    let input = std::fs::read(path)
        // TODO this error should be better
        .or(Err(Error::IO))?;
    decode(input.as_ref())
}
