//! Reed-Solomon BCH encoder and decoder suitable for `no_std` environment.
//!
//! This library implements block encoder and decoder: error correction code is appended to original data.
//!
//! # Example
//! ```rust
//! extern crate reed_solomon;
//!
//! use reed_solomon::Encoder;
//! use reed_solomon::Decoder;
//!
//! fn main() {
//!     let data = b"Hello World!";
//!
//!     // Length of error correction code
//!     let ecc_len = 8;
//!
//!     // Create encoder and decoder with
//!     let enc = Encoder::new(ecc_len);
//!     let dec = Decoder::new(ecc_len);
//!
//!     // Encode data
//!     let encoded = enc.encode(&data[..]);
//!
//!     // Simulate some transmission errors
//!     let mut corrupted = *encoded;
//!     for i in 0..4 {
//!         corrupted[i] = 0x0;
//!     }
//!
//!     // Try to recover data
//!     let known_erasures = [0];
//!     let recovered = dec.correct(&mut corrupted, Some(&known_erasures)).unwrap();
//!
//!     let orig_str = std::str::from_utf8(data).unwrap();
//!     let recv_str = std::str::from_utf8(recovered.data()).unwrap();
//!
//!     println!("message:               {:?}", orig_str);
//!     println!("original data:         {:?}", data);
//!     println!("error correction code: {:?}", encoded.ecc());
//!     println!("corrupted:             {:?}", corrupted);
//!     println!("repaired:              {:?}", recv_str);
//! }
//! ```
//!
//! # Unsafe
//! This library uses some slices indexind that is boundary checked.
//!
//! You can disable checks with library feature `unsafe_indexing`, 
//! then unsafe `Slice::get_inchecked()` would be utilized to improve speed where unchecked indexing
//! is considered safe and LLVM cannot drop boundary checks.
//!
//! # Bandwidth
//! Software implementation is relatively slow because general purpose processors do not support
//! Galois field arithmetic operations. For example, Galois field multiply requires test for 0,
//! two table look-ups, modulo add, and anti-log table look-up.
//!
//! Besides this performance bound, current implementation is not very optimal
//! and performs some unnecessary memcpys.
//!
//! Encoder bandwidth using one Sandy Bridge core operating on 2.8 `GHz`:
//! <style type="text/css">
//! .tg  {border-collapse:collapse;border-spacing:0;border-color:#ccc;}
//! .tg td{font-family:Arial, sans-serif;font-size:14px;padding:10px 5px;border-style:solid;border-width:1px;overflow:hidden;word-break:normal;border-color:#ccc;color:#333;background-color:#fff;}
//! .tg th{font-family:Arial, sans-serif;font-size:14px;font-weight:normal;padding:10px 5px;border-style:solid;border-width:1px;overflow:hidden;word-break:normal;border-color:#ccc;color:#333;background-color:#f0f0f0;}
//! .tg .tg-baqh{text-align:center;vertical-align:top}
//! </style>
//! <table class="tg">
//!   <tr>
//!     <th class="tg-baqh">data<br></th>
//!     <th class="tg-baqh">ecc</th>
//!     <th class="tg-baqh">bandwidth<br></th>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh">251</td>
//!     <td class="tg-baqh">4</td>
//!     <td class="tg-baqh">115.20 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh">239</td>
//!     <td class="tg-baqh">16</td>
//!     <td class="tg-baqh">37.76 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh">223</td>
//!     <td class="tg-baqh">32</td>
//!     <td class="tg-baqh">19.59 MB/s<br></td>
//!   </tr>
//! </table>
//!
//! Decoder bandwidth using one Sandy Bridge core operating on 2.8 `GHz`:
//! <style type="text/css">
//! .tg  {border-collapse:collapse;border-spacing:0;border-color:#ccc;}
//! .tg td{font-family:Arial, sans-serif;font-size:14px;padding:10px 5px;border-style:solid;border-width:1px;overflow:hidden;word-break:normal;border-color:#ccc;color:#333;background-color:#fff;}
//! .tg th{font-family:Arial, sans-serif;font-size:14px;font-weight:normal;padding:10px 5px;border-style:solid;border-width:1px;overflow:hidden;word-break:normal;border-color:#ccc;color:#333;background-color:#f0f0f0;}
//! .tg .tg-uqo3{background-color:#efefef;text-align:center;vertical-align:top}
//! .tg .tg-baqh{text-align:center;vertical-align:top}
//! </style>
//! <table class="tg">
//!   <tr>
//!     <th class="tg-baqh">data<br></th>
//!     <th class="tg-baqh">ecc</th>
//!     <th class="tg-baqh">errors</th>
//!     <th class="tg-baqh">bandwidth</th>
//!   </tr>
//!   <tr>
//!     <td class="tg-uqo3">251</td>
//!     <td class="tg-uqo3">4</td>
//!     <td class="tg-uqo3">0<br></td>
//!     <td class="tg-uqo3">49 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh"><br></td>
//!     <td class="tg-baqh"><br></td>
//!     <td class="tg-baqh">1</td>
//!     <td class="tg-baqh">16.91 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh"><br></td>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh">2</td>
//!     <td class="tg-baqh">15.90 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-uqo3">239</td>
//!     <td class="tg-uqo3">16</td>
//!     <td class="tg-uqo3">0</td>
//!     <td class="tg-uqo3">10.75 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh">1</td>
//!     <td class="tg-baqh">4.86 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh">8</td>
//!     <td class="tg-baqh">3.81 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-uqo3">223</td>
//!     <td class="tg-uqo3">32</td>
//!     <td class="tg-uqo3">0</td>
//!     <td class="tg-uqo3">4.80 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh">1</td>
//!     <td class="tg-baqh">2.32 MB/s<br></td>
//!   </tr>
//!   <tr>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh"></td>
//!     <td class="tg-baqh">16</td>
//!     <td class="tg-baqh">1.82 MB/s<br></td>
//!   </tr>
//! </table>

#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![warn(missing_docs, missing_debug_implementations,
        trivial_casts, trivial_numeric_casts,
        unstable_features)]

#![no_std]

const POLYNOMIAL_MAX_LENGTH: usize = 256;

#[macro_use]
mod macros;
mod gf;
mod encoder;
mod decoder;
mod buffer;

pub use encoder::Encoder;
pub use decoder::Decoder;
pub use decoder::DecoderError;
pub use buffer::Buffer;
