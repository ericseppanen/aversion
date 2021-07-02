//! Provides a `DataSink` and `DataSource` using the CBOR format.

use crate::group::{DataSink, DataSource};
use crate::util::FixedHeader;
use serde::de::DeserializeOwned;
use std::io::{self, Read, Write};
use thiserror::Error;

/// Errors that may occur while reading or writing CborData data.
#[derive(Debug, Error)]
pub enum CborDataError {
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

impl From<serde_cbor::Error> for CborDataError {
    fn from(e: serde_cbor::Error) -> Self {
        use serde_cbor::error::Category;

        match e.classify() {
            Category::Io => CborDataError::Io(None),
            Category::Syntax => CborDataError::Serializer,
            Category::Data => CborDataError::Serializer,
            Category::Eof => CborDataError::Eof,
        }
    }
}

impl From<io::Error> for CborDataError {
    fn from(e: io::Error) -> Self {
        CborDataError::Io(Some(e))
    }
}

/// A [`DataSource`] and/or [`DataSink`] using the CBOR serialization format.
///
/// [`CborData`] works with any type that implements [`Read`] or [`Write`].
/// That includes files, network sockets, and memory buffers.
///
/// It implements the [`DataSource`] trait if the inner type implements [`Read`],
/// and implements the [`DataSink`] trait if the inner type implements [`Write`].
///
/// [`Read`]: std::io::Read
/// [`Write`]: std::io::Write
///
pub struct CborData<RW> {
    inner: RW,
}

impl<RW> CborData<RW> {
    /// Create a new `CborData`.
    pub fn new(reader: RW) -> Self {
        CborData { inner: reader }
    }

    /// Consume the `CborData`, returning the inner data type.
    pub fn into_inner(self) -> RW {
        self.inner
    }
}

impl<R> DataSource for CborData<R>
where
    R: Read,
{
    type Error = CborDataError;
    type Header = FixedHeader;

    fn read_header(&mut self) -> Result<FixedHeader, CborDataError> {
        Ok(FixedHeader::deserialize_from(&mut self.inner)?)
    }

    fn read_message<T>(&mut self) -> Result<T, CborDataError>
    where
        T: DeserializeOwned,
    {
        let msg: T = serde_cbor::from_reader(&mut self.inner)?;
        Ok(msg)
    }
}

impl<W> DataSink for CborData<W>
where
    W: Write,
{
    type Error = CborDataError;
    type Header = FixedHeader;

    fn write_header(&mut self, header: &FixedHeader) -> Result<(), CborDataError> {
        Ok(header.serialize_into(&mut self.inner)?)
    }

    fn write_bare_message<T>(&mut self, msg: &T) -> Result<(), CborDataError>
    where
        T: serde::Serialize,
    {
        Ok(serde_cbor::to_writer(&mut self.inner, msg)?)
    }
}
