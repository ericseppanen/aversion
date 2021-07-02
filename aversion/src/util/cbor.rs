//! Provides a `DataSink` and `DataSource` using the CBOR format.

use crate::group::{DataSink, DataSource};
use crate::util::BasicHeader;
use crate::{MessageId, Versioned};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryInto;
use std::io::{self, Cursor, Read, Write};
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
    type Header = BasicHeader;

    fn read_header(&mut self) -> Result<BasicHeader, CborDataError> {
        Ok(BasicHeader::deserialize_from(&mut self.inner)?)
    }

    fn read_message<T>(&mut self, header: &BasicHeader) -> Result<T, CborDataError>
    where
        T: DeserializeOwned,
    {
        // Construct a reader over the exact message length specified
        // in the message header.
        let reader = &mut self.inner;
        let mut subreader = reader.take(header.msg_len.into());
        let msg: T = serde_cbor::from_reader(&mut subreader)?;
        Ok(msg)
    }
}

impl<W> DataSink for CborData<W>
where
    W: Write,
{
    type Error = CborDataError;

    fn write_message<T>(&mut self, msg: &T) -> Result<(), CborDataError>
    where
        T: Serialize + Versioned,
        T::Base: MessageId,
    {
        // Serialize the message first, then the header (which needs
        // the serialized message length.
        let msg_buf = Vec::<u8>::new();
        let mut cursor = Cursor::new(msg_buf);
        serde_cbor::to_writer(&mut cursor, msg)?;
        let msg_buf = cursor.into_inner();
        let msg_len: u32 = msg_buf.len().try_into().expect("usize to u32");
        let header = BasicHeader::for_msg(msg, msg_len);
        header.serialize_into(&mut self.inner)?;
        self.inner.write_all(&msg_buf)?;
        Ok(())
    }
}
