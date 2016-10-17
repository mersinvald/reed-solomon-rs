use ::gf::poly::Polynom;
use core::ops::{Deref, DerefMut};

/// Buffer for block encoded data
/// # Example
/// ```rust
/// use reed_solomon::Buffer;
///
/// let buffer = Buffer::from_slice(&[1, 2, 3, 4], 2);
/// assert_eq!(&[1, 2], buffer.data());
/// assert_eq!(&[3, 4], buffer.ecc());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Buffer {
    poly: Polynom,
    data_len: usize,
}

impl Buffer {
    /// Create buffer from internal polynom
    pub fn from_polynom(poly: Polynom, data_len: usize) -> Self {
        Buffer {
            poly: poly,
            data_len: data_len,
        }
    }

    /// Create buffer from [u8] slice
    pub fn from_slice(slice: &[u8], data_len: usize) -> Self {
        Buffer {
            poly: Polynom::from(slice),
            data_len: data_len,
        }
    }

    /// Slice with data of encoded block
    pub fn data(&self) -> &[u8] {
        &self[..self.data_len]
    }

    /// Slice with error correction core of encoced block
    pub fn ecc(&self) -> &[u8] {
        &self[self.data_len..]
    }

    /// Add byte string to the end of buffer
    pub fn append(&mut self, rhs: &[u8]) {
        let ofst = self.len();
        self.set_length(ofst + rhs.len());
        for (i, rhs_x) in rhs.iter().enumerate() {
            self[i + ofst] = *rhs_x;
        }
    }
}

impl Deref for Buffer {
    type Target = Polynom;
    fn deref(&self) -> &Self::Target {
        &self.poly
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.poly
    }
}

impl From<Polynom> for Buffer {
    fn from(p: Polynom) -> Buffer {
        Buffer {
            data_len: p.len(),
            poly: p,
        }
    }
}
