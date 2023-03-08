use crate::alloc::{
    string::{String, ToString},
    vec::Vec,
};
use crate::errors::OscError;
use crate::types::{OscBundle, OscMessage, OscPacket, OscTime, OscType, Result};

use byteorder::{BigEndian, ByteOrder};

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
    match *packet {
        OscPacket::Message(ref msg) => encode_message(msg),
        OscPacket::Bundle(ref bundle) => encode_bundle(bundle),
    }
}

fn encode_message(msg: &OscMessage) -> Result<Vec<u8>> {
    let mut msg_bytes: Vec<u8> = Vec::new();

    encode_string_into(&msg.addr, &mut msg_bytes);
    let mut type_tags: Vec<char> = vec![','];
    let mut arg_bytes: Vec<u8> = Vec::new();

    for arg in &msg.args {
        let (bytes, tags): (Option<Vec<u8>>, String) = encode_arg(arg)?;

        type_tags.extend(tags.chars());
        if let Some(data) = bytes {
            arg_bytes.extend(data);
        }
    }

    msg_bytes.extend(encode_string(type_tags.into_iter().collect::<String>()));
    if !arg_bytes.is_empty() {
        msg_bytes.extend(arg_bytes);
    }
    Ok(msg_bytes)
}

fn encode_bundle(bundle: &OscBundle) -> Result<Vec<u8>> {
    let mut bundle_bytes: Vec<u8> = Vec::new();
    encode_string_into("#bundle", &mut bundle_bytes);

    match encode_arg(&OscType::Time(bundle.timetag))? {
        (Some(x), _) => {
            bundle_bytes.extend(x.into_iter());
        }
        (None, _) => {
            return Err(OscError::BadBundle("Missing time tag!".to_string()));
        }
    }

    if bundle.content.is_empty() {
        return Ok(bundle_bytes);
    }

    for packet in &bundle.content {
        match *packet {
            OscPacket::Message(ref m) => {
                let msg = encode_message(m)?;
                let mut msg_size = [0u8; 4];
                BigEndian::write_u32(&mut msg_size, msg.len() as u32);
                bundle_bytes.extend(msg_size.iter().copied().chain(msg.into_iter()));
            }
            OscPacket::Bundle(ref b) => {
                let bdl = encode_bundle(b)?;
                let mut bdl_size = [0u8; 4];
                BigEndian::write_u32(&mut bdl_size, bdl.len() as u32);
                bundle_bytes.extend(bdl_size.iter().copied().chain(bdl.into_iter()));
            }
        }
    }

    Ok(bundle_bytes)
}

fn encode_arg(arg: &OscType) -> Result<(Option<Vec<u8>>, String)> {
    match *arg {
        OscType::Int(ref x) => {
            let mut bytes = vec![0u8; 4];
            BigEndian::write_i32(&mut bytes, *x);
            Ok((Some(bytes), "i".into()))
        }
        OscType::Long(ref x) => {
            let mut bytes = vec![0u8; 8];
            BigEndian::write_i64(&mut bytes, *x);
            Ok((Some(bytes), "h".into()))
        }
        OscType::Float(ref x) => {
            let mut bytes = vec![0u8; 4];
            BigEndian::write_f32(&mut bytes, *x);
            Ok((Some(bytes), "f".into()))
        }
        OscType::Double(ref x) => {
            let mut bytes = vec![0u8; 8];
            BigEndian::write_f64(&mut bytes, *x);
            Ok((Some(bytes), "d".into()))
        }
        OscType::Char(ref x) => {
            let mut bytes = vec![0u8; 4];
            BigEndian::write_u32(&mut bytes, *x as u32);
            Ok((Some(bytes), "c".into()))
        }
        OscType::String(ref x) => {
            let mut bytes = vec![0u8; 0];
            encode_string_into(x, &mut bytes);
            Ok((Some(bytes), "s".into()))
        },
        OscType::Blob(ref x) => {
            let padded_blob_length: usize = pad(x.len() as u64) as usize;
            let mut bytes = vec![0u8; 4 + padded_blob_length];
            // write length
            BigEndian::write_i32(&mut bytes[..4], x.len() as i32);
            for (i, v) in x.iter().enumerate() {
                bytes[i + 4] = *v;
            }
            Ok((Some(bytes), "b".into()))
        }
        OscType::Time(time) => Ok((Some(encode_time_tag(time)), "t".into())),
        OscType::Midi(ref x) => Ok((Some(vec![x.port, x.status, x.data1, x.data2]), "m".into())),
        OscType::Color(ref x) => Ok((Some(vec![x.red, x.green, x.blue, x.alpha]), "r".into())),
        OscType::Bool(ref x) => {
            if *x {
                Ok((None, "T".into()))
            } else {
                Ok((None, "F".into()))
            }
        }
        OscType::Nil => Ok((None, "N".into())),
        OscType::Inf => Ok((None, "I".into())),
        OscType::Array(ref x) => {
            let mut bytes = vec![0u8; 0];
            let mut type_tags = String::from("[");
            for v in x.content.iter() {
                match encode_arg(v) {
                    Ok((Some(other_bytes), other_type_tags)) => {
                        bytes.extend(other_bytes);
                        type_tags.push_str(&other_type_tags);
                    }
                    Ok((None, other_type_tags)) => {
                        type_tags.push_str(&other_type_tags);
                    }
                    Err(err) => return Err(err),
                }
            }
            type_tags.push(']');
            Ok((Some(bytes), type_tags))
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

fn encode_time_tag(time: OscTime) -> Vec<u8> {
    let mut bytes = vec![0u8; 8];
    BigEndian::write_u32(&mut bytes[..4], time.seconds);
    BigEndian::write_u32(&mut bytes[4..], time.fractional);
    bytes
}

#[test]
fn test_pad() {
    assert_eq!(4, pad(4));
    assert_eq!(8, pad(5));
    assert_eq!(8, pad(6));
    assert_eq!(8, pad(7));
}
