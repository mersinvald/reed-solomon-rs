use ::gf::poly::Polynom;
use core::ops::{Deref, DerefMut};

pub struct Buffer {
    poly: Polynom,
    data_len: usize,
}

impl Buffer {
    pub fn from_polynom(poly: Polynom, data_len: usize) -> Self {
        Buffer {
            poly: poly,
            data_len: data_len,
        }
    }

    pub fn from_slice(slice: &[u8], data_len: usize) -> Self {
        Buffer {
            poly: Polynom::from(slice),
            data_len: data_len,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self[..self.data_len]
    }

    pub fn ecc(&self) -> &[u8] {
        &self[self.data_len..]
    }

    pub fn append(&mut self, rhs: &[u8]) {
        let ofst = self.len();
        self.length += rhs.len();
        for i in 0..rhs.len() {
            self[i + ofst] = rhs[i];
        }
    }

    pub fn into_poly(self) -> Polynom {
        self.poly
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
