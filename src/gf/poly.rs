use ::gf;
use ::buffer::Buffer;
use core::ops;
use core::cmp::max;

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
    pub fn copy_from_slice(slice: &[u8]) -> Polynom {
        debug_assert!(slice.len() <= ::POLYNOMIAL_MAX_LENGTH);
        let mut new_array = [0; ::POLYNOMIAL_MAX_LENGTH];

        new_array[0..slice.len()].copy_from_slice(slice);

        Polynom {
            array: new_array,
            length: slice.len(),
        }
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
}

impl Clone for Polynom {
    #[inline]
    fn clone(&self) -> Polynom {
        *self
    }
}

impl ops::Deref for Polynom {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.array[0..self.len()]
    }
}

impl<'a> Into<Polynom> for &'a [u8] {
    #[inline]
    fn into(self) -> Polynom {
        Polynom::copy_from_slice(&self)
    }
}

impl<'a> Into<Polynom> for Buffer {
    #[inline]
    fn into(self) -> Polynom {
        self.into_poly()
    }
}

impl ops::DerefMut for Polynom {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len();
        &mut self.array[0..len]
    }
}

/// Multiplication by scalar
impl ops::MulAssign<u8> for Polynom {
    #[inline]
    fn mul_assign(&mut self, x: u8) {
        for i in 0..self.length {
            self[i] = gf::mul(self[i], x);
        }
    }
}

impl ops::Mul<u8> for Polynom {
    type Output = Polynom;
    #[inline]
    fn mul(mut self, x: u8) -> Self::Output {
        self *= x;
        self
    }
}

/// Polynomials addition
impl ops::Add for Polynom {
    type Output = Polynom;
    #[inline]
    fn add(mut self, rhs: Polynom) -> Self::Output {
        self += rhs;
        self
    }
}

impl ops::AddAssign for Polynom {
    fn add_assign(&mut self, rhs: Polynom) {
        let mut result = Polynom::with_length(max(self.len(), rhs.len()));

        for i in 0..self.len() {
            let index = i + result.len() - self.len();
            uncheck_mut!(result[index]) = self[i];
        }

        for i in 0..rhs.len() {
            let index = i + result.len() - rhs.len();
            uncheck_mut!(result[index]) ^= rhs[i];
        }

        self.array = result.array;
        self.shrink(result.len());
    }
}

/// Polynomials multiplication
impl ops::Mul for Polynom {
    #[inline]
    type Output = Polynom;
    fn mul(mut self, rhs: Polynom) -> Self::Output {
        self *= rhs;
        self
    }
}

impl ops::MulAssign for Polynom {
    fn mul_assign(&mut self, rhs: Polynom) {
        let mut result = Polynom::with_length(self.len() + rhs.len() - 1);

        for j in 0..rhs.len() {
            for i in 0..self.len() {
                uncheck_mut!(result[i + j]) ^= gf::mul(self[i], rhs[j]);
            }
        }

        self.array = result.array;
        self.shrink(result.len());
    }
}

/// Polynomial division
// Note: ops::Div can't be used because division returns (quotient, remainder)
impl Polynom {
    pub fn div(&self, rhs: &Polynom) -> (Polynom, Polynom) {
        let mut result = Polynom::copy_from_slice(self);

        // If divisor's degree (len-1) is bigger, all dividend is a remainder
        let divisor_degree = rhs.len() - 1;
        if self.len() < divisor_degree {
            return (Polynom::new(), result);
        }

        for i in 0..(self.len() - divisor_degree) {
            let coef = result[i];
            if coef != 0 {
                for j in 1..rhs.len() {
                    if rhs[j] != 0 {
                        uncheck_mut!(result[i + j]) ^= gf::mul(rhs[j], coef);
                    }
                }
            }
        }

        let separator = self.len() - (rhs.len() - 1);

        // Quotient is after separator
        let remainder = Polynom::copy_from_slice(&result[separator..]);

        // And reminder is before separator, so just shrink to it
        result.shrink(separator);

        (result, remainder)
    }

    fn shrink(&mut self, new_len: usize) {
        if new_len < self.len() {
            for i in new_len..self.len() {
                self[i] = 0;
            }
        }

        self.length = new_len;
    }
}

impl Polynom {
    /// Evaluate a polynomial in GF(2^p) given the value for x.
    #[inline]
    pub fn eval(&self, x: u8) -> u8 {
        let mut y = self[0];
        for px in self.iter().skip(1) {
            y = gf::mul(y, x) ^ px;
        }
        y
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reverse() {
        let poly = polynom![5, 4, 3, 2, 1, 0];
        for (i, x) in poly.reverse().iter().enumerate() {
            assert_eq!(i, *x as usize);
        }
    }

    #[test]
    fn scale() {
        let poly = polynom![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let answer = [0, 3, 6, 5, 12, 15, 10, 9, 24, 27];
        assert_eq!(answer, *(poly * 3));
    }

    #[test]
    fn add() {
        let px = polynom![0, 5, 10, 15, 20];
        let py = polynom![3, 9, 17, 24, 75];
        assert_eq!([3, 12, 27, 23, 95], *(px + py));

        let px = polynom![0, 5, 10];
        let py = polynom![3, 9, 17, 24, 75];

        assert_eq!([3, 9, 17, 29, 65], *(px + py));
        assert_eq!([3, 9, 17, 29, 65], *(py + px));
    }

    #[test]
    fn mul() {
        let px = polynom![0, 5, 10, 15, 20];
        let py = polynom![3, 9, 17, 24, 75];
        assert_eq!([0, 15, 51, 30, 153, 193, 53, 115, 245], *(px * py));

        let px = polynom![0, 5, 10];
        let py = polynom![3, 9, 17, 24, 75];

        assert_eq!([0, 15, 51, 15, 210, 138, 244], *(px * py));
        assert_eq!([0, 15, 51, 15, 210, 138, 244], *(py * px));
    }

    #[test]
    fn div() {
        let px = polynom![0, 5, 10, 15, 20];
        let py = polynom![3, 9, 17, 24, 75];

        let (q, r) = px.div(&py);
        assert_eq!([0], *q);
        assert_eq!([5, 10, 15, 20], *r);

        let (q, r) = py.div(&px);
        assert_eq!([3], *q);
        assert_eq!([6, 15, 9, 119], *r);

        let px = polynom![0, 5, 10];
        let py = polynom![3, 9, 17, 24, 75];

        let empty: [u8; 0] = [];
        let (q, r) = px.div(&py);

        assert_eq!(empty, *q);
        assert_eq!([0, 5, 10], *r);

        let (q, r) = py.div(&px);
        assert_eq!([3, 6, 17], *q);
        assert_eq!([113, 225], *r);
    }

    #[test]
    fn eval() {
        let p = polynom![0, 5, 10, 15, 20];
        let tests = [4, 7, 21, 87, 35, 255];
        let answers = [213, 97, 132, 183, 244, 92];

        for i in 0..tests.len() {
            assert_eq!(answers[i], p.eval(tests[i]));
        }
    }
}

use core::fmt;
impl fmt::Debug for Polynom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", &self[..])
    }
}