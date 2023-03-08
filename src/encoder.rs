use crate::alloc::{
    string::String,
    vec::Vec,
};
use crate::types::{OscBundle, OscMessage, OscPacket, OscTime, OscType, Result};

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
pub fn encode(packet: &OscPacket) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    encode_into(packet, &mut bytes)?;
    Ok(bytes)
}

pub fn encode_into(packet: &OscPacket, out: &mut Vec<u8>) -> Result<()> {
    match *packet {
        OscPacket::Message(ref msg) => encode_message(msg, out),
        OscPacket::Bundle(ref bundle) => encode_bundle(bundle, out),
    }
}

fn encode_message(msg: &OscMessage, out: &mut Vec<u8>) -> Result<()> {
    encode_string_into(&msg.addr, out);

    let tag_start = out.len();
    out.push(b',');
    for arg in &msg.args {
        encode_arg_type(arg, out)?;
    }
    let tag_len = out.len() - tag_start;

    let new_len = tag_start + pad(tag_len as u64 + 1) as usize;
    out.resize(new_len, 0);

    for arg in &msg.args {
        encode_arg_data(arg, out)?;
    }

    Ok(())
}

fn encode_bundle(bundle: &OscBundle, out: &mut Vec<u8>) -> Result<()> {
    encode_string_into("#bundle", out);
    encode_time_tag_into(&bundle.timetag, out);

    for packet in &bundle.content {
        match *packet {
            OscPacket::Message(ref m) => {
                let len_start = out.len();
                out.extend(0u32.to_be_bytes());

                let msg_start = out.len();
                encode_message(m, out)?;
                let msg_len = out.len() - msg_start;

                out[len_start..msg_start].copy_from_slice(&(msg_len as u32).to_be_bytes());
            }
            OscPacket::Bundle(ref b) => {
                let len_start = out.len();
                out.extend(0u32.to_be_bytes());

                let bundle_start = out.len();
                encode_bundle(b, out)?;
                let bundle_len = out.len() - bundle_start;

                out[len_start..bundle_start].copy_from_slice(&(bundle_len as u32).to_be_bytes());
            }
        }
    }

    Ok(())
}

fn encode_arg_data(arg: &OscType, out: &mut Vec<u8>) -> Result<()> {
    match *arg {
        OscType::Int(x) => {
            out.extend(x.to_be_bytes());
            Ok(())
        }
        OscType::Long(x) => {
            out.extend(x.to_be_bytes());
            Ok(())
        }
        OscType::Float(x) => {
            out.extend(x.to_be_bytes());
            Ok(())
        }
        OscType::Double(x) => {
            out.extend(x.to_be_bytes());
            Ok(())
        }
        OscType::Char(x) => {
            out.extend((x as u32).to_be_bytes());
            Ok(())
        }
        OscType::String(ref x) => {
            encode_string_into(x, out);
            Ok(())
        }
        OscType::Blob(ref x) => {
            let padded_blob_length: usize = pad(x.len() as u64) as usize;
            out.reserve(4 + padded_blob_length);
            out.extend((x.len() as u32).to_be_bytes());

            let new_len = out.len() + padded_blob_length;
            out.extend(x);
            out.resize(new_len, 0);

            Ok(())
        }
        OscType::Time(ref time) => {
            encode_time_tag_into(time, out);
            Ok(())
        }
        OscType::Midi(ref x) => {
            out.extend([x.port, x.status, x.data1, x.data2]);
            Ok(())
        }
        OscType::Color(ref x) => {
            out.extend([x.red, x.green, x.blue, x.alpha]);
            Ok(())
        }
        OscType::Bool(_) => Ok(()),
        OscType::Nil => Ok(()),
        OscType::Inf => Ok(()),
        OscType::Array(ref x) => {
            for v in &x.content {
                encode_arg_data(v, out)?;
            }
            Ok(())
        }
    }
}

fn encode_arg_type(arg: &OscType, out: &mut Vec<u8>) -> Result<()> {
    match *arg {
        OscType::Int(_) => {
            out.push(b'i');
            Ok(())
        }
        OscType::Long(_) => {
            out.push(b'h');
            Ok(())
        }
        OscType::Float(_) => {
            out.push(b'f');
            Ok(())
        }
        OscType::Double(_) => {
            out.push(b'd');
            Ok(())
        }
        OscType::Char(_) => {
            out.push(b'c');
            Ok(())
        }
        OscType::String(_) => {
            out.push(b's');
            Ok(())
        },
        OscType::Blob(_) => {
            out.push(b'b');
            Ok(())
        }
        OscType::Time(_) => {
            out.push(b't');
            Ok(())
        },
        OscType::Midi(_) => {
            out.push(b'm');
            Ok(())
        }
        OscType::Color(_) => {
            out.push(b'r');
            Ok(())
        }
        OscType::Bool(x) => {
            out.push(if x { b'T' } else { b'F' });
            Ok(())
        }
        OscType::Nil => {
            out.push(b'N');
            Ok(())
        }
        OscType::Inf => {
            out.push(b'I');
            Ok(())
        }
        OscType::Array(ref x) => {
            out.push(b'[');
            for v in &x.content {
                encode_arg_type(v, out)?;
            }
            out.push(b']');
            Ok(())
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
pub fn encode_string_into<S: AsRef<str>>(s: S, out: &mut Vec<u8>) {
    let s = s.as_ref();

    let padded_len = pad(s.len() as u64 + 1) as usize;
    out.reserve(padded_len);

    let new_len = out.len() + padded_len;
    out.extend(s.as_bytes());
    out.resize(new_len, 0u8);
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

fn encode_time_tag_into(time: &OscTime, out: &mut Vec<u8>) {
    out.extend(time.seconds.to_be_bytes());
    out.extend(time.fractional.to_be_bytes());
}

#[test]
fn test_pad() {
    assert_eq!(4, pad(4));
    assert_eq!(8, pad(5));
    assert_eq!(8, pad(6));
    assert_eq!(8, pad(7));
}
