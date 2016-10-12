#![feature(test)]
extern crate test;
extern crate reed_solomon;

use reed_solomon::encoder::Encoder;

const DATA_SIZE: usize = 200;
const ECC_SIZE: usize = 40;

#[bench]
fn name(b: &mut test::Bencher) {
    let mut data = [0; DATA_SIZE];
    for i in 0..DATA_SIZE {
        data[i] = i as u8;
    }

    let encoder = Encoder::new(DATA_SIZE, ECC_SIZE);

    b.iter(|| {
        encoder.encode(&data[..]);
    })
}
