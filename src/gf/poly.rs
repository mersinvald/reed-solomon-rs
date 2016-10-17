#[derive(Copy)]
pub struct Polynom {
    pub array: [u8; ::POLYNOMIAL_MAX_LENGTH],
    pub length: usize,
}

impl Polynom {
    #[inline]
    pub fn new() -> Polynom {
        Polynom {
            array: [0; ::POLYNOMIAL_MAX_LENGTH],
            length: 0,
        }
    }

    #[inline]
    pub fn with_length(len: usize) -> Polynom {
        let mut p = Polynom::new();
        p.length = len;
        p
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn reverse(mut self) -> Self {
        (*self).reverse();
        self
    }

    #[inline]
    pub fn push(&mut self, x: u8) {
        self.array[self.length] = x;
        self.length += 1;
    }

    #[inline]
    pub fn shrink(&mut self, new_len: usize) {
        if new_len < self.len() {
            for x in self.iter_mut().skip(new_len) {
                *x = 0;
            }
        }

        self.length = new_len;
    }
}

impl Clone for Polynom {
    #[inline]
    fn clone(&self) -> Polynom {
        *self
    }
}

impl Default for Polynom {
    fn default() -> Self {
        Self::new()
    }
}

use core::ops::Deref;
impl Deref for Polynom {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        let len = self.len();
        &self.array[0..len]
    }
}

use core::ops::DerefMut;
impl DerefMut for Polynom {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len();
        &mut self.array[0..len]
    }
}

impl<'a> From<&'a [u8]> for Polynom {
    #[inline]
    fn from(slice: &'a [u8]) -> Polynom {
        debug_assert!(slice.len() <= ::POLYNOMIAL_MAX_LENGTH);
        let mut new_array = [0; ::POLYNOMIAL_MAX_LENGTH];

        new_array[0..slice.len()].copy_from_slice(slice);

        Polynom {
            array: new_array,
            length: slice.len(),
        }
    }
}

use core::fmt;
impl fmt::Debug for Polynom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", &self[..])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn push() {
        let mut poly = polynom![];
        for i in 0..10 {
            poly.push(i);
            for j in 0..(i as usize) {
                assert!(poly[j] == j as u8);
            }
        }
    }

    #[test]
    fn reverse() {
        let poly = polynom![5, 4, 3, 2, 1, 0];
        for (i, x) in poly.reverse().iter().enumerate() {
            assert_eq!(i, *x as usize);
        }
    }

    #[test]
    fn shrink() {
        let mut poly = polynom![1; 16];
        poly.shrink(2);

        for i in 0..2 {
            assert_eq!(poly.array[i], 1);
        }

        for i in 2..256 {
            assert_eq!(poly.array[i], 0);
        }
    }
}