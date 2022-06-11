#![feature(test)]

extern crate test;
use p92000::fcall;
use std::borrow::Cow;
use test::Bencher;

#[bench]
fn encode_walk(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(4096);
    let fcall = fcall::Fcall::Twalk(fcall::Twalk {
        fid: 123,
        new_fid: 345,
        wnames: ["abc", "def", "hij"]
            .iter()
            .map(|x| Cow::from(*x))
            .collect(),
    });
    let tagged_fcall = fcall::TaggedFcall {
        tag: 123,
        fcall: fcall,
    };
    b.iter(|| tagged_fcall.encode_to_buf(&mut buf).unwrap());
}

#[bench]
fn decode_walk(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(4096);
    let fcall = fcall::Fcall::Twalk(fcall::Twalk {
        fid: 123,
        new_fid: 345,
        wnames: ["abc", "def", "hij"]
            .iter()
            .map(|x| Cow::from(*x))
            .collect(),
    });
    let tagged_fcall = fcall::TaggedFcall {
        tag: 123,
        fcall: fcall,
    };
    tagged_fcall.encode_to_buf(&mut buf).unwrap();
    b.iter(|| fcall::TaggedFcall::decode(&buf).unwrap());
}
