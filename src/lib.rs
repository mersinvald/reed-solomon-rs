#![feature(question_mark)]
#![no_std]

const POLYNOMIAL_MAX_LENGTH: usize = 256;

macro_rules! polynom {
    [$value:expr; $count:expr] => {
        $crate::gf::poly::Polynom::copy_from_slice(&[$value; $count])
    }; 

    [$( $value:expr ),* ] => {
        $crate::gf::poly::Polynom::copy_from_slice(&[$($value, )*])
    };
}

mod gf;
mod encoder;
mod decoder;
mod buffer;

pub use encoder::Encoder;
pub use decoder::Decoder;
pub use decoder::ReedSolomonError;
pub use buffer::Buffer;
