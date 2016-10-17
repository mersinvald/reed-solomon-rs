extern crate reed_solomon;
extern crate rustc_serialize;

use reed_solomon::Encoder;
use reed_solomon::Decoder;

struct Generator {
    pub num: u8
}

impl Generator {
    fn new() -> Generator {
        Generator {
            num: 2
        }
    }
}

impl Iterator for Generator {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.num = self.num.rotate_right(1);
        Some(self.num)
    }
}

use std::thread;
use std::time::Duration;
use std::sync::mpsc;

// Returns MB/s
fn encoder_bandwidth(data_len: usize, ecc_len: usize) -> f32 { 
     // Measure encoding bandwidth
    let (tx, thr_rx) = mpsc::channel();
    let (thr_tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let generator = Generator::new();
        let encoder = Encoder::new(ecc_len);

        let buffer: Vec<u8> = generator.take(data_len).collect();
        let mut bytes = 0;
        while thr_rx.try_recv().is_err() {
            encoder.encode(&buffer);
            bytes += data_len;
        }

        thr_tx.send(bytes).unwrap();
    });

    thread::sleep(Duration::from_secs(1));

    tx.send(()).unwrap();
    let bytes = rx.recv().unwrap();
    let kbytes = (bytes / 1024) as f32;
    kbytes / 1024.0
}

// Returns MB/s
fn decoder_bandwidth(data_len: usize, ecc_len: usize, errors: usize) -> f32 {
     // Measure decoder bandwidth
    let (tx, thr_rx) = mpsc::channel();
    let (thr_tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let generator = Generator::new();
        let encoder = Encoder::new(ecc_len);
        let decoder = Decoder::new(ecc_len);

        let buffer: Vec<u8> = generator.take(data_len).collect();
        let mut encoded = encoder.encode(&buffer);
        for x in encoded.iter_mut().take(errors) {
            *x = 0;
        } 

        let mut bytes = 0;
        while thr_rx.try_recv().is_err() {
            if decoder.is_corrupted(&encoded) {
                decoder.correct(&mut encoded, None).unwrap();
            }            
            bytes += data_len;
        }

        thr_tx.send(bytes).unwrap();
    });

    thread::sleep(Duration::from_secs(1));

    tx.send(()).unwrap();
    let bytes = rx.recv().unwrap();
    let kbytes = (bytes / 1024) as f32;
    kbytes / 1024.0
} 

#[derive(RustcEncodable)]
struct BenchResult {
    data_len: usize,
    ecc_len: usize,
    encoder: EncoderResult,
    decoder: Vec<DecoderResult>
} 

#[derive(RustcEncodable)]
struct EncoderResult {
    bandwidth: f32
}

#[derive(RustcEncodable)]
struct DecoderResult {
    errors: usize,
    bandwidth: f32
}

fn main() {
    let results: Vec<BenchResult> = [(251, 4), (239, 16), (223, 32)].iter().map(|case| {
        let data_len = case.0;
        let ecc_len = case.1;

        BenchResult {
            data_len: data_len,
            ecc_len: ecc_len,
            encoder: EncoderResult {
                bandwidth: encoder_bandwidth(data_len, ecc_len),
            },
            decoder: (0..(ecc_len / 2) + 1).map(|e| DecoderResult {
                errors: e,
                bandwidth: decoder_bandwidth(data_len, ecc_len, e)
            }).collect()
        }
    }).collect();

    let json = rustc_serialize::json::encode(&results).unwrap();
    println!("{}", json);
}
