use ::gf::poly_math::*;
use ::gf::poly::Polynom;
use ::buffer::Buffer;
use ::gf;

/// Reed-Solomon BCH encoder
#[derive(Debug)]
pub struct Encoder {
    generator: Polynom,
}

impl Encoder {
    /// Constructs a new `Encoder` and calculates generator polynomial of given `ecc_len`.
    ///
    /// # Example
    /// ```rust
    /// use reed_solomon::Encoder;
    ///
    /// let encoder = Encoder::new(8);
    /// ```
    pub fn new(ecc_len: usize) -> Self {
        Encoder { generator: generator_poly(ecc_len) }
    }

    /// Encodes passed `&[u8]` slice and returns `Buffer` with result and `ecc` offset.
    ///
    /// # Example
    /// ```rust
    /// use reed_solomon::Encoder;
    ///
    /// let data = "Hello World".as_bytes();
    /// let encoder = Encoder::new(8);
    ///
    /// let encoded = encoder.encode(&data);
    ///
    /// println!("whole: {:?}", &encoded[..]);
    /// println!("data:  {:?}", encoded.data());
    /// println!("ecc:   {:?}", encoded.ecc());
    /// ```
    pub fn encode(&self, data: &[u8]) -> Buffer {
        let mut data = Polynom::from(data);
        let data_len = data.len();

        data.length += self.generator.len() - 1;

        let (_, rem) = data.div(&self.generator);

        data.length = data_len;

        let mut data = Buffer::from(data);
        data.append(&rem);
        data
    }
}

fn generator_poly(ecclen: usize) -> Polynom {
    let mut gen = polynom![1];
    let mut mm = [1, 0];
    for i in 0..ecclen {
        mm[1] = gf::pow(2, i as i32);
        gen = gen.mul(&mm);
    }
    gen
}


#[cfg(test)]
mod tests {
    #[test]
    fn generator_poly() {
        let answers =
            [polynom![1, 3, 2],
             polynom![1, 15, 54, 120, 64],
             polynom![1, 255, 11, 81, 54, 239, 173, 200, 24],
             polynom![1, 59, 13, 104, 189, 68, 209, 30, 8, 163, 65, 41, 229, 98, 50, 36, 59],
             polynom![1, 116, 64, 52, 174, 54, 126, 16, 194, 162, 33, 33, 157, 176, 197, 225, 12,
                      59, 55, 253, 228, 148, 47, 179, 185, 24, 138, 253, 20, 142, 55, 172, 88],
             polynom![1, 193, 10, 255, 58, 128, 183, 115, 140, 153, 147, 91, 197, 219, 221, 220,
                      142, 28, 120, 21, 164, 147, 6, 204, 40, 230, 182, 14, 121, 48, 143, 77,
                      228, 81, 85, 43, 162, 16, 195, 163, 35, 149, 154, 35, 132, 100, 100, 51,
                      176, 11, 161, 134, 208, 132, 244, 176, 192, 221, 232, 171, 125, 155, 228,
                      242, 245]];

        let mut ecclen = 2;
        for i in 0..6 {
            assert_eq!(*answers[i], *super::generator_poly(ecclen));
            ecclen *= 2;
        }
    }

    #[test]
    fn encode() {
        let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29];
        let ecc = [99, 26, 219, 193, 9, 94, 186, 143];

        let encoder = super::Encoder::new(ecc.len());
        let encoded = encoder.encode(&data[..]);

        assert_eq!(data, encoded.data());
        assert_eq!(ecc, encoded.ecc());
    }

}
