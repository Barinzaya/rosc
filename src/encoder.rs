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

/// Appends the given string `s` to the given Vec `out`,
/// adding 1-4 null bytes such that the length of the result
/// is a multiple of 4.
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

pub trait Output {
    type Err;
    type Placeholder;

    fn allocate(&mut self, size: usize) -> Result<Self::Placeholder, Self::Err>;
    fn rewrite(&mut self, mark: Self::Placeholder, data: &[u8]) -> Result<(), Self::Err>;
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err>;

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
