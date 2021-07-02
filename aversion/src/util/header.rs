use crate::group::GroupHeader;
use crate::{MessageId, Versioned};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

/// A header that can be serialized into a fixed-size buffer.
///
/// This header does not use serde; it serializes to a binary
/// (big-endian) array of 4 bytes.
#[derive(Debug, Clone, Copy)]
pub struct TinyHeader {
    pub msg_id: u16,
    pub msg_ver: u16,
}

impl TinyHeader {
    pub fn new(msg_id: u16, msg_ver: u16) -> Self {
        TinyHeader { msg_id, msg_ver }
    }

    pub fn for_msg<T>(_msg: &T) -> Self
    where
        T: Versioned,
        T::Base: MessageId,
    {
        TinyHeader {
            msg_id: T::Base::MSG_ID,
            msg_ver: T::VER,
        }
    }

    /// Deserialize a header from a `Read` stream.
    pub fn deserialize_from(r: &mut impl Read) -> Result<Self, io::Error> {
        let msg_id = r.read_u16::<BigEndian>()?;
        let msg_ver = r.read_u16::<BigEndian>()?;
        Ok(TinyHeader { msg_id, msg_ver })
    }

    /// Deserialize a header from a 4-byte slice.
    pub fn deserialize(buf: impl AsRef<[u8; 4]>) -> Self {
        // Use a &[u8] as the Read stream.
        let mut buf: &[u8] = buf.as_ref();
        // No io::Error is possible, since we're doing no actual IO.
        Self::deserialize_from(&mut buf).unwrap()
    }

    /// Serialize a header into a `Write` stream.
    pub fn serialize_into(self, w: &mut impl Write) -> Result<(), io::Error> {
        w.write_u16::<BigEndian>(self.msg_id)?;
        w.write_u16::<BigEndian>(self.msg_ver)?;
        Ok(())
    }

    /// Serialize a header into a 4-byte array.
    pub fn serialize(self) -> [u8; 4] {
        let mut buf = [0u8; 4];
        // Use a &[u8] as the Write stream.
        let mut cursor: &mut [u8] = buf.as_mut();
        // No io::Error is possible, since we're doing no actual IO.
        self.serialize_into(&mut cursor).unwrap();
        buf
    }
}

impl GroupHeader for TinyHeader {
    fn msg_id(&self) -> u16 {
        self.msg_id
    }

    fn msg_ver(&self) -> u16 {
        self.msg_ver
    }
}

/// A header that can be serialized into a fixed-size buffer.
///
/// This header does not use serde; it serializes to a binary
/// (big-endian) array of 8 bytes.
///
/// Unlike [`TinyHeader`], this header includes a message length
/// value, which may make deserialization easier or more efficient.
#[derive(Debug, Clone, Copy)]
pub struct BasicHeader {
    pub msg_id: u16,
    pub msg_ver: u16,
    pub msg_len: u32,
}

impl BasicHeader {
    pub fn new(msg_id: u16, msg_ver: u16, msg_len: u32) -> Self {
        BasicHeader {
            msg_id,
            msg_ver,
            msg_len,
        }
    }

    pub fn for_msg<T>(_msg: &T, msg_len: u32) -> Self
    where
        T: Versioned,
        T::Base: MessageId,
    {
        BasicHeader {
            msg_id: T::Base::MSG_ID,
            msg_ver: T::VER,
            msg_len,
        }
    }

    /// Deserialize a header from a `Read` stream.
    pub fn deserialize_from(r: &mut impl Read) -> Result<Self, io::Error> {
        let msg_id = r.read_u16::<BigEndian>()?;
        let msg_ver = r.read_u16::<BigEndian>()?;
        let msg_len = r.read_u32::<BigEndian>()?;
        Ok(BasicHeader {
            msg_id,
            msg_ver,
            msg_len,
        })
    }

    /// Deserialize a header from a 4-byte slice.
    pub fn deserialize(buf: impl AsRef<[u8; 4]>) -> Self {
        // Use a &[u8] as the Read stream.
        let mut buf: &[u8] = buf.as_ref();
        // No io::Error is possible, since we're doing no actual IO.
        Self::deserialize_from(&mut buf).unwrap()
    }

    /// Serialize a header into a `Write` stream.
    pub fn serialize_into(self, w: &mut impl Write) -> Result<(), io::Error> {
        w.write_u16::<BigEndian>(self.msg_id)?;
        w.write_u16::<BigEndian>(self.msg_ver)?;
        w.write_u32::<BigEndian>(self.msg_len)?;
        Ok(())
    }

    /// Serialize a header into a 4-byte array.
    pub fn serialize(self) -> [u8; 4] {
        let mut buf = [0u8; 4];
        // Use a &[u8] as the Write stream.
        let mut cursor: &mut [u8] = buf.as_mut();
        // No io::Error is possible, since we're doing no actual IO.
        self.serialize_into(&mut cursor).unwrap();
        buf
    }
}

impl GroupHeader for BasicHeader {
    fn msg_id(&self) -> u16 {
        self.msg_id
    }

    fn msg_ver(&self) -> u16 {
        self.msg_ver
    }
}
