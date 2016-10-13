#![feature(question_mark)]
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
pub use decoder::ReedSolomonError;
pub use buffer::Buffer;
