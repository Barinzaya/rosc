#![feature(test)]
extern crate rosc;
extern crate test;

use rosc::*;
use self::test::Bencher;

#[bench]
fn bench_encode_args_bool(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Bools".into(),
				args: (0..1000).into_iter().map(|x| OscType::Bool((x % 2) == 1))
					.collect(),
			}),
		],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_double(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Doubles".into(),
				args: (0..1000).into_iter().map(|x| OscType::Double(x as f64))
					.collect(),
			}),
		],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_float(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Floats".into(),
				args: (0..1000).into_iter().map(|x| OscType::Float(x as f32))
					.collect(),
			}),
		],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_int(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Ints".into(),
				args: (0..1000).into_iter().map(OscType::Int)
					.collect(),
			}),
		],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_long(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Longs".into(),
				args: (0..1000).into_iter().map(OscType::Long)
					.collect(),
			}),
		],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_nil(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Nils".into(),
				args: (0..1000).into_iter().map(|_| OscType::Nil)
					.collect(),
			}),
		],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_string(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Strings".into(),
				args: (0..1000).into_iter().map(|x| OscType::String(x.to_string()))
					.collect(),
			}),
		],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_messages(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Message".into(),
				args: vec![],
			})
		; 1000],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}
