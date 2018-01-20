# Reed-Solomon BCH
[![license](https://img.shields.io/github/license/mashape/apistatus.svg)]()
[![Build Status](https://travis-ci.org/mersinvald/reed-solomon-rs.svg?branch=master)](https://travis-ci.org/mersinvald/reed-solomon-rs)
[![Crates.io](https://img.shields.io/crates/v/reed-solomon.svg)](https://crates.io/crates/reed-solomon)
[![Documentation](https://docs.rs/reed-solomon/badge.svg)](https://docs.rs/reed-solomon)

Reed-Solomon BCH encoder and decoder implemented in Rust.
This is a port of python implementation from [Wikiversity](https://en.wikiversity.org/wiki/Reed–Solomon_codes_for_coders)

## Setup 

```toml
[dependencies]
reed-solomon = "0.2"
```

```rust
extern crate reed_solomon
```

## Example

```rust
extern crate reed_solomon;

use reed_solomon::Encoder;
use reed_solomon::Decoder;

fn main() {
    let data = b"Hello World!";

    // Length of error correction code
    let ecc_len = 8;

    // Create encoder and decoder with 
    let enc = Encoder::new(ecc_len);
    let dec = Decoder::new(ecc_len);

    // Encode data
    let encoded = enc.encode(&data[..]);

    // Simulate some transmission errors
    let mut corrupted = *encoded;
    for i in 0..4 {
        corrupted[i] = 0x0;
    }

    // Try to recover data
    let known_erasures = [0];
    let recovered = dec.correct(&mut corrupted, Some(&known_erasures)).unwrap();

    let orig_str = std::str::from_utf8(data).unwrap();
    let recv_str = std::str::from_utf8(recovered.data()).unwrap();

    println!("message:               {:?}", orig_str);
    println!("original data:         {:?}", data);
    println!("error correction code: {:?}", encoded.ecc());
    println!("corrupted:             {:?}", corrupted);
    println!("repaired:              {:?}", recv_str);
}
```
