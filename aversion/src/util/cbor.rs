//! Provides a `DataSource` module using the CBOR format.

use crate::group::DataSource;
use crate::util::FixedHeader;
use serde::de::DeserializeOwned;
use std::io::{self, Read};
use thiserror::Error;

/// Errors that may occur while reading or writing CborSource data.
#[derive(Debug, Error)]
pub enum CborSourceError {
    /// A `std::io::Error` occurred while reading or writing data.
    #[error("IO Error")]
    Io(Option<io::Error>),
    /// An error occurred while serializing or deserializing data.
    #[error("Serialize/Deserialize Error")]
    Serializer,
    /// An EOF happened while attempting to read data.
    #[error("Premature EOF")]
    Eof,
}

impl From<serde_cbor::Error> for CborSourceError {
    fn from(e: serde_cbor::Error) -> Self {
        use serde_cbor::error::Category;

        match e.classify() {
            Category::Io => CborSourceError::Io(None),
            Category::Syntax => CborSourceError::Serializer,
            Category::Data => CborSourceError::Serializer,
            Category::Eof => CborSourceError::Eof,
        }
    }
}

impl From<io::Error> for CborSourceError {
    fn from(e: io::Error) -> Self {
        CborSourceError::Io(Some(e))
    }
}

/// A [`DataSource`] using the CBOR serialization format.
///
/// [`CborSource`] works with any type that implements [`Read`].
/// That includes files, network sockets, and memory buffers.
///
/// [`Read`]: std::io::Read
///
pub struct CborSource<R> {
    reader: R,
}

impl<R> CborSource<R>
where
    R: Read,
{
    /// Create a new `CborSource`.
    pub fn new(reader: R) -> Self {
        CborSource { reader }
    }

    /// Consume the `CborSource`, returning the original `Read` object.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R> DataSource for CborSource<R>
where
    R: Read,
{
    type Error = CborSourceError;
    type Header = FixedHeader;

    fn read_header(&mut self) -> Result<Self::Header, Self::Error> {
        Ok(FixedHeader::deserialize_from(&mut self.reader)?)
    }

    fn read_message<T>(&mut self) -> Result<T, Self::Error>
    where
        T: DeserializeOwned,
    {
        let msg: T = serde_cbor::from_reader(&mut self.reader)?;
        Ok(msg)
    }
}
