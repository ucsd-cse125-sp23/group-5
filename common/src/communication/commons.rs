use log::debug;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};

pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:7878";
pub const CSE125_SERVER_ADDR: &str = "128.54.70.15:2333";
pub const DEFAULT_MOUSE_MOVEMENT_INTERVAL: u64 = 5; // 5ms
pub const MAX_WIND_CHARGE: u32 = 10;
pub const ONE_CHARGE: u32 = 1;
pub const FLAG_XZ: (f32, f32) = (0.0, 0.0);
pub const FLAG_RADIUS: f32 = 2.0;
pub const FLAG_Z_BOUND: (Option<f32>, Option<f32>) = (Some(-10.0), Some(0.0));
pub const WINNING_THRESHOLD: f32 = 20.0;
pub const DECAY_RATE: f32 = 1.0 / 3.0;

/// Trait for something that can be converted to bytes (&[u8])
pub trait Serialize {
    /// Serialize to a `Write`able buffer
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()>;
}
/// Trait for something that can be converted from bytes (&[u8])
pub trait Deserialize {
    /// The type that this deserializes to
    type Output;

    /// Deserialize from a `Read`able buffer
    fn deserialize(buf: &mut impl Read) -> io::Result<Self::Output>;
}

/// Abstracted Protocol that wraps a TcpStream and manages
/// sending & receiving of messages
pub struct Protocol {
    reader: io::BufReader<TcpStream>,
    stream: TcpStream,
}

impl Protocol {
    /// Wrap a TcpStream with Protocol
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            reader: io::BufReader::new(stream.try_clone()?),
            stream,
        })
    }

    /// Establish a connection, wrap stream in BufReader/Writer
    pub fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest)?;
        debug!("Connecting to {}", dest);
        Self::with_stream(stream)
    }

    /// Serialize a message to the server and write it to the TcpStream
    pub fn send_message(&mut self, message: &impl Serialize) -> io::Result<()> {
        message.serialize(&mut self.stream)?;
        self.stream.flush()
    }

    /// Read a message from the inner TcpStream
    ///
    /// NOTE: Will block until there's data to read (or deserialize fails with io::ErrorKind::Interrupted)
    ///       so only use when a message is expected to arrive
    pub fn read_message<T: Deserialize>(&mut self) -> io::Result<T::Output> {
        T::deserialize(&mut self.reader)
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            reader: io::BufReader::new(self.stream.try_clone()?),
            stream: self.stream.try_clone()?,
        })
    }
}

/// This module provides functions for reading and writing data with
/// a length-prefix format.
///
/// It supports reading and writing byte arrays, strings, and JSON-encoded
/// values, with length information stored in a big-endian 32-bit unsigned
/// integer.
pub mod prefix_len {
    use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
    use std::io;
    use std::io::{Read, Write};

    pub fn extract_bytes(buf: &mut impl Read) -> io::Result<Vec<u8>> {
        let length = buf.read_u32::<NetworkEndian>()?;
        let mut bytes = vec![0u8; length as usize];
        buf.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    pub fn extract_string(buf: &mut impl Read) -> io::Result<String> {
        let bytes = extract_bytes(buf)?;
        String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid utf8"))
    }

    pub fn extract_bincode<T: for<'a> serde::Deserialize<'a>>(
        buf: &mut impl Read,
    ) -> io::Result<T> {
        let bincode = extract_bytes(buf)?;
        let value = bincode::deserialize(&bincode)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(value)
    }

    pub fn write_bytes(buf: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
        buf.write_u32::<NetworkEndian>(bytes.len() as u32)?;
        buf.write_all(bytes)?;
        Ok(())
    }

    pub fn write_string(buf: &mut impl Write, string: &str) -> io::Result<()> {
        write_bytes(buf, string.as_bytes())
    }

    pub fn write_bincode(buf: &mut impl Write, obj: &impl serde::Serialize) -> io::Result<()> {
        let bytes =
            bincode::serialize(obj).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write_bytes(buf, &bytes)
    }
}
