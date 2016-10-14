#![feature(test)]
#![feature(inclusive_range_syntax)]

extern crate reed_solomon;
extern crate test;

use reed_solomon::Encoder;
use reed_solomon::Decoder;

struct Generator {
    pub bytes: u64,
    pub num: u8
}

impl Generator {
    fn new() -> Generator {
        Generator {
            bytes: 0,
            num: 2
        }
    }
}

impl Iterator for Generator {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.bytes += 1;
        self.num = self.num.rotate_right(1);
        Some(self.num)
    }
}

use std::thread;
use std::time::Duration;
use std::sync::mpsc;

const DATA_LEN: usize = 223;
const ECC_LEN: usize = 32;

// Returns MB/s
fn encoder_bandwidth() -> f32 { 
     // Measure encoding bandwidth
    let (tx, thr_rx) = mpsc::channel();
    let (thr_tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let mut generator = Generator::new();
        let encoder = Encoder::new(ECC_LEN);

        let mut buffer = [0; DATA_LEN];
        while thr_rx.try_recv().is_err() {
            for i in 0..DATA_LEN {
                buffer[i] = generator.next().unwrap(); 
            }

            encoder.encode(&buffer);
        }

        thr_tx.send(generator.bytes).unwrap();
    });

    thread::sleep(Duration::from_secs(1));

    tx.send(()).unwrap();
    let bytes = rx.recv().unwrap();
    let kbytes = (bytes / 1024) as f32;
    kbytes / 1024.0
}

fn decoder_bandwidth(errors: usize) -> f32 {
     // Measure decoder bandwidth
    let (tx, thr_rx) = mpsc::channel();
    let (thr_tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let mut generator = Generator::new();
        let encoder = Encoder::new(ECC_LEN);
        let decoder = Decoder::new(ECC_LEN);

        let mut buffer = [0; DATA_LEN];
        for i in 0..DATA_LEN {
            buffer[i] = generator.next().unwrap(); 
        }

        let mut encoded = test::black_box(encoder.encode(&buffer));
        for i in 0..errors {
            encoded[i] = 0;
        } 

        let mut bytes = 0;
        while thr_rx.try_recv().is_err() {
            decoder.decode(&encoded, None).unwrap();            
            bytes += DATA_LEN;
        }

        thr_tx.send(bytes).unwrap();
    });

    thread::sleep(Duration::from_secs(1));

    tx.send(()).unwrap();
    let bytes = rx.recv().unwrap();
    let kbytes = (bytes / 1024) as f32;
    kbytes / 1024.0
} 

fn main() {
    println!("Reed-Solomon(data: {}, ecc: {})", DATA_LEN, ECC_LEN);
    println!("Encoder bandwidth: {0:.2} MB/s", encoder_bandwidth());
    for i in 0...(ECC_LEN / 2) {
        println!("Decoder bandwidth, {0} errors: {1:.2} MB/s", i, decoder_bandwidth(i));
    }
}