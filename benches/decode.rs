#![feature(test)]
extern crate test;
extern crate reed_solomon;

use reed_solomon::Encoder;
use reed_solomon::Decoder;

const DATA_SIZE: usize = 200;
const ECC_SIZE: usize = 40;

#[bench]
fn decode(b: &mut test::Bencher) {
    let mut data = [0; DATA_SIZE];
    for i in 0..DATA_SIZE {
        data[i] = i as u8;
    }

    let encoder = Encoder::new(ECC_SIZE);
    let decoder = Decoder::new(ECC_SIZE);
    let encoded = encoder.encode(&data[..]);

    b.iter(|| {
        decoder.decode(&encoded, None).unwrap();
    })
}
