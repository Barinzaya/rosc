#![feature(test)]
extern crate rosc;
extern crate test;

use rosc::*;
use self::test::Bencher;

#[bench]
fn bench_encode_args_bool(b: &mut Bencher) {
	let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Bools".into(),
        args: (0..1000).into_iter().map(|x| OscType::Bool((x % 2) == 1))
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_double(b: &mut Bencher) {
	let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Doubles".into(),
        args: (0..1000).into_iter().map(|x| OscType::Double(x as f64))
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_float(b: &mut Bencher) {
	let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Floats".into(),
        args: (0..1000).into_iter().map(|x| OscType::Float(x as f32))
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_int(b: &mut Bencher) {
	let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Ints".into(),
        args: (0..1000).into_iter().map(OscType::Int)
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_long(b: &mut Bencher) {
	let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Longs".into(),
        args: (0..1000).into_iter().map(OscType::Long)
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_nil(b: &mut Bencher) {
	let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Nils".into(),
        args: (0..1000).into_iter().map(|_| OscType::Nil)
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_string(b: &mut Bencher) {
	let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Strings".into(),
        args: (0..1000).into_iter().map(|x| OscType::String(x.to_string()))
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_bundles(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Bundle(OscBundle {
				timetag: (0, 0).into(),
				content: vec![],
			})
		; 1000],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_bundles_into_new(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Bundle(OscBundle {
				timetag: (0, 0).into(),
				content: vec![],
			})
		; 1000],
	});

    b.iter(|| rosc::encoder::encode_into(&packet, &mut Vec::new()).unwrap());
}

#[bench]
fn bench_encode_bundles_into_reuse(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Bundle(OscBundle {
				timetag: (0, 0).into(),
				content: vec![],
			})
		; 1000],
	});

	let mut buffer = Vec::new();
    b.iter(|| {
		buffer.clear();
		rosc::encoder::encode_into(&packet, &mut buffer).unwrap()
	});
}

#[bench]
fn bench_encode_huge(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Message".into(),
				args: vec![
					4i32.into(),
					42i64.into(),
					3.1415926f32.into(),
					3.14159265359f64.into(),
					"String".into(),
					(0..1024).into_iter().map(|x| x as u8).collect::<Vec<u8>>().into(),
					(123, 456).into(),
					'c'.into(),
					false.into(),
					true.into(),
					OscType::Nil,
					OscType::Inf,
					OscMidiMessage {
						port: 4,
						status: 41,
						data1: 42,
						data2: 129,
					}.into(),
					OscColor {
						red: 255,
						green: 192,
						blue: 42,
						alpha: 13,
					}.into(),
					OscArray {
						content: vec![
							42i32.into(),
							OscArray {
								content: vec![1.23.into(), 3.21.into()],
							}.into(),
							"Another String".into(),
						],
					}.into(),
				],
			})
		; 1000],
	});

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_huge_into_new(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Message".into(),
				args: vec![
					4i32.into(),
					42i64.into(),
					3.1415926f32.into(),
					3.14159265359f64.into(),
					"String".into(),
					(0..1024).into_iter().map(|x| x as u8).collect::<Vec<u8>>().into(),
					(123, 456).into(),
					'c'.into(),
					false.into(),
					true.into(),
					OscType::Nil,
					OscType::Inf,
					OscMidiMessage {
						port: 4,
						status: 41,
						data1: 42,
						data2: 129,
					}.into(),
					OscColor {
						red: 255,
						green: 192,
						blue: 42,
						alpha: 13,
					}.into(),
					OscArray {
						content: vec![
							42i32.into(),
							OscArray {
								content: vec![1.23.into(), 3.21.into()],
							}.into(),
							"Another String".into(),
						],
					}.into(),
				],
			})
		; 1000],
	});

    b.iter(|| rosc::encoder::encode_into(&packet, &mut Vec::new()).unwrap());
}

#[bench]
fn bench_encode_huge_into_reuse(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Message".into(),
				args: vec![
					4i32.into(),
					42i64.into(),
					3.1415926f32.into(),
					3.14159265359f64.into(),
					"String".into(),
					(0..1024).into_iter().map(|x| x as u8).collect::<Vec<u8>>().into(),
					(123, 456).into(),
					'c'.into(),
					false.into(),
					true.into(),
					OscType::Nil,
					OscType::Inf,
					OscMidiMessage {
						port: 4,
						status: 41,
						data1: 42,
						data2: 129,
					}.into(),
					OscColor {
						red: 255,
						green: 192,
						blue: 42,
						alpha: 13,
					}.into(),
					OscArray {
						content: vec![
							42i32.into(),
							OscArray {
								content: vec![1.23.into(), 3.21.into()],
							}.into(),
							"Another String".into(),
						],
					}.into(),
				],
			})
		; 1000],
	});

	let mut buffer = Vec::new();
    b.iter(|| {
		buffer.clear();
		rosc::encoder::encode_into(&packet, &mut buffer).unwrap()
	});
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

#[bench]
fn bench_encode_messages_into_new(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Message".into(),
				args: vec![],
			})
		; 1000],
	});

    b.iter(|| rosc::encoder::encode_into(&packet, &mut Vec::new()).unwrap());
}

#[bench]
fn bench_encode_messages_into_reuse(b: &mut Bencher) {
	let packet = OscPacket::Bundle(OscBundle {
		timetag: (0, 0).into(),
		content: vec![
			OscPacket::Message(OscMessage {
				addr: "/OSC/Message".into(),
				args: vec![],
			})
		; 1000],
	});

	let mut buffer = Vec::new();
    b.iter(|| {
		buffer.clear();
		rosc::encoder::encode_into(&packet, &mut buffer).unwrap()
	});
}
