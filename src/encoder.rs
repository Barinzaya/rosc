use crate::alloc::{
    string::String,
    vec::Vec,
};
use crate::types::{OscBundle, OscMessage, OscPacket, OscTime, OscType};

/// Takes a reference to an OSC packet and returns
/// a byte vector on success. If the packet was invalid
/// an `OscError` is returned.
///
/// # Example
///
/// ```
/// use rosc::{OscPacket,OscMessage,OscType};
/// use rosc::encoder;
///
/// let packet = OscPacket::Message(OscMessage{
///         addr: "/greet/me".to_string(),
///         args: vec![OscType::String("hi!".to_string())]
///     }
/// );
/// assert!(encoder::encode(&packet).is_ok())
/// ```
pub fn encode(packet: &OscPacket) -> crate::types::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let _ = encode_into(packet, &mut bytes);
    Ok(bytes)
}

/// Takes a reference to an OSC packet and writes the
/// encoded bytes to the given output. On success, the
/// number of bytes written will be returned. If an error
/// occurs during encoding, encoding will stop and the
/// error will be returned. Note that in that case, the
/// output may have been partially written!
///
/// # Example
///
/// ```
/// use rosc::{OscPacket,OscMessage,OscType};
/// use rosc::encoder;
///
/// let mut bytes = Vec::new();
/// let packet = OscPacket::Message(OscMessage{
///         addr: "/greet/me".to_string(),
///         args: vec![OscType::String("hi!".to_string())]
///     }
/// );
/// assert!(encoder::encode_into(&packet, &mut bytes).is_ok())
/// ```
pub fn encode_into<O: Output>(packet: &OscPacket, out: &mut O) -> Result<usize, O::Err> {
    match *packet {
        OscPacket::Message(ref msg) => encode_message(msg, out),
        OscPacket::Bundle(ref bundle) => encode_bundle(bundle, out),
    }
}

fn encode_message<O: Output>(msg: &OscMessage, out: &mut O) -> Result<usize, O::Err> {
    let mut written = encode_string_into(&msg.addr, out)?;

    written += out.write(b",")?;
    for arg in &msg.args {
        written += encode_arg_type(arg, out)?;
    }

    let padding = pad(written as u64 + 1) as usize - written;
    written += out.write(&[0u8; 4][..padding])?;

    for arg in &msg.args {
        written += encode_arg_data(arg, out)?;
    }

    Ok(written)
}

fn encode_bundle<O: Output>(bundle: &OscBundle, out: &mut O) -> Result<usize, O::Err> {
    let mut written = encode_string_into("#bundle", out)?;
    written += encode_time_tag_into(&bundle.timetag, out)?;

    for packet in &bundle.content {
        match *packet {
            OscPacket::Message(ref m) => {
                let len_place = out.allocate(4)?;
                written += 4;

                let msg_len = encode_message(m, out)?;
                written += msg_len;

                out.rewrite(len_place, &(msg_len as u32).to_be_bytes())?;
            }
            OscPacket::Bundle(ref b) => {
                let len_place = out.allocate(4)?;
                written += 4;

                let bundle_len = encode_bundle(b, out)?;
                written += bundle_len;

                out.rewrite(len_place, &(bundle_len as u32).to_be_bytes())?;
            }
        }
    }

    Ok(written)
}

fn encode_arg_data<O: Output>(arg: &OscType, out: &mut O) -> Result<usize, O::Err> {
    match *arg {
        OscType::Int(x) => out.write(&x.to_be_bytes()),
        OscType::Long(x) => out.write(&x.to_be_bytes()),
        OscType::Float(x) => out.write(&x.to_be_bytes()),
        OscType::Double(x) => out.write(&x.to_be_bytes()),
        OscType::Char(x) => out.write(&(x as u32).to_be_bytes()),
        OscType::String(ref x) => encode_string_into(x, out),
        OscType::Blob(ref x) => {
            let padded_blob_length: usize = pad(x.len() as u64) as usize;
            let padding = padded_blob_length - x.len();

            out.reserve(4 + padded_blob_length)?;
            out.write(&(x.len() as u32).to_be_bytes())?;
            out.write(x)?;

            if padding > 0 {
                out.write(&[0u8; 3][..padding])?;
            }

            Ok(4 + padded_blob_length)
        }
        OscType::Time(ref time) => encode_time_tag_into(time, out),
        OscType::Midi(ref x) => out.write(&[x.port, x.status, x.data1, x.data2]),
        OscType::Color(ref x) => out.write(&[x.red, x.green, x.blue, x.alpha]),
        OscType::Bool(_) => Ok(0),
        OscType::Nil => Ok(0),
        OscType::Inf => Ok(0),
        OscType::Array(ref x) => {
            let mut written = 0;
            for v in &x.content {
                written += encode_arg_data(v, out)?;
            }
            Ok(written)
        }
    }
}

fn encode_arg_type<O: Output>(arg: &OscType, out: &mut O) -> Result<usize, O::Err> {
    match *arg {
        OscType::Int(_) => out.write(b"i"),
        OscType::Long(_) => out.write(b"h"),
        OscType::Float(_) => out.write(b"f"),
        OscType::Double(_) => out.write(b"d"),
        OscType::Char(_) => out.write(b"c"),
        OscType::String(_) => out.write(b"s"),
        OscType::Blob(_) => out.write(b"b"),
        OscType::Time(_) => out.write(b"t"),
        OscType::Midi(_) => out.write(b"m"),
        OscType::Color(_) => out.write(b"r"),
        OscType::Bool(x) => out.write(if x { b"T" } else { b"F" }),
        OscType::Nil => out.write(b"N"),
        OscType::Inf => out.write(b"I"),
        OscType::Array(ref x) => {
            let mut written = out.write(b"[")?;

            for v in &x.content {
                written += encode_arg_type(v, out)?;
            }

            written += out.write(b"]")?;
            Ok(written)
        }
    }
}

/// Null terminates the byte representation of string `s` and
/// adds null bytes until the length of the result is a
/// multiple of 4.
pub fn encode_string<S: Into<String>>(s: S) -> Vec<u8> {
    let mut bytes: Vec<u8> = s.into().into_bytes();

    let new_len = pad(bytes.len() as u64 + 1) as usize;
    bytes.resize(new_len, 0u8);

    bytes
}

/// Writes the given string `s` to the given Output, adding
/// 1-4 null bytes such that the length of the result is a
/// multiple of 4.
pub fn encode_string_into<S: AsRef<str>, O: Output>(s: S, out: &mut O) -> Result<usize, O::Err> {
    let s = s.as_ref();

    let padded_len = pad(s.len() as u64 + 1) as usize;
    out.reserve(padded_len)?;

    let padding = padded_len - s.len();
    out.write(s.as_bytes())?;
    out.write(&[0u8; 4][..padding])?;
    Ok(s.len() + padding)
}

/// Returns the position padded to 4 bytes.
///
/// # Example
///
/// ```
/// use rosc::encoder;
///
/// let pos: u64 = 10;
/// assert_eq!(12u64, encoder::pad(pos))
/// ```
pub fn pad(pos: u64) -> u64 {
    match pos % 4 {
        0 => pos,
        d => pos + (4 - d),
    }
}

fn encode_time_tag_into<O: Output>(time: &OscTime, out: &mut O) -> Result<usize, O::Err> {
    out.write(&time.seconds.to_be_bytes())?;
    out.write(&time.fractional.to_be_bytes())?;
    Ok(8)
}

#[test]
fn test_pad() {
    assert_eq!(4, pad(4));
    assert_eq!(8, pad(5));
    assert_eq!(8, pad(6));
    assert_eq!(8, pad(7));
}

/// A trait for values that can receive encoded OSC output
/// via `encode_into`. This allows more flexibility in how
/// the output is handled, including reusing part of an
/// existing buffer or writing directly to an external sink
/// (e.g. a file).
///
/// Implementations are currently provided for this trait:
/// - `Vec<u8>`: Data will be appended to the end of the Vec.
/// - `NullOutput`: Data is not written anywhere.
///   Potentially useful for calculating the size of a
///   packet without writing it anywhere.
///
/// Note that the OSC encoder will write output in small
/// pieces (as small as a single byte), so the output should
/// be buffered if write calls have a large overhead (e.g.
/// writing to a file).
pub trait Output {
    /// The error type which is returned from Output functions.
    type Err;

    /// A placeholder for data that is to be filled in after
    /// additional data is written.
    type Placeholder;

    /// Allocates space in the output for data to be
    /// retroactively written later, returning a
    /// `Placeholder` that can be used to fill in this data
    /// later (with `rewrite`).
    fn allocate(&mut self, size: usize) -> Result<Self::Placeholder, Self::Err>;

    /// Rewrites data that was previously allocated with
    /// `allocate`. The `Placeholder` is moved in, so any
    /// allocated space may only be written once. The data
    /// given should be of the exact size that was given to
    /// `allocate`.
    fn rewrite(&mut self, mark: Self::Placeholder, data: &[u8]) -> Result<(), Self::Err>;

    /// Writes a block of data to the output. The size of
    /// the data is return on success.
    ///
    /// Note that, unlike `std::io::Writo::write`, this
    /// function is expected to write all of the given data.
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err>;

    /// Reserves space in the output to write at least the
    /// given number of bytes.
    ///
    /// This is used as an optimization prior to writing
    /// certain data loads, but should not be depended on
    /// for correct output.
    fn reserve(&mut self, _size: usize) -> Result<(), Self::Err> { Ok(()) }
}

impl<T: Output> Output for &mut T {
    type Err = T::Err;
    type Placeholder = T::Placeholder;

    fn allocate(&mut self, size: usize) -> Result<Self::Placeholder, Self::Err> {
        T::allocate(*self, size)
    }

    fn reserve(&mut self, size: usize) -> Result<(), Self::Err> {
        T::reserve(*self, size)
    }

    fn rewrite(&mut self, mark: Self::Placeholder, data: &[u8]) -> Result<(), Self::Err> {
        T::rewrite(*self, mark, data)
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err> {
        T::write(*self, data)
    }
}

impl Output for Vec<u8> {
    type Err = core::convert::Infallible;
    type Placeholder = (usize, usize);

    fn allocate(&mut self, size: usize) -> Result<Self::Placeholder, Self::Err> {
        let start = self.len();
        let end = start + size;

        self.resize(end, 0);
        Ok((start, end))
    }

    fn reserve(&mut self, size: usize) -> Result<(), Self::Err> {
        Vec::reserve(self, size);
        Ok(())
    }

    fn rewrite(&mut self, (start, end): Self::Placeholder, data: &[u8]) -> Result<(), Self::Err> {
        self[start..end].copy_from_slice(data);
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err> {
        self.extend(data);
        Ok(data.len())
    }
}

/// An implementation of `Output` that does not write the
/// data anywhere.
///
/// Intended for use as an `Output` to pre-calculate sizes
/// without actually writing any data.
#[derive(Clone, Copy, Debug)]
pub struct NullOutput;

impl Output for NullOutput {
    type Err = core::convert::Infallible;
    type Placeholder = ();

    #[inline(always)]
    fn allocate(&mut self, _: usize) -> Result<Self::Placeholder, Self::Err> {
        Ok(())
    }

    #[inline(always)]
    fn rewrite(&mut self, _: Self::Placeholder, _: &[u8]) -> Result<(), Self::Err> {
        Ok(())
    }

    #[inline(always)]
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err> {
        Ok(data.len())
    }
}
