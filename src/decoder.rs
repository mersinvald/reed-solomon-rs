use ::gf::poly_math::*;
use ::gf::poly::Polynom;
use ::buffer::Buffer;
use ::gf;

#[derive(Debug)]
pub enum ReedSolomonError {
    TooManyErrors,
}

#[derive(Debug)]
pub struct Decoder {
    ecc_len: usize,
}

impl Decoder {
    pub fn new(ecc_len: usize) -> Self {
        Decoder { ecc_len: ecc_len }
    }

    pub fn decode<'a>(&'a self, msg: &'a [u8], erase_pos: Option<&[u8]>) -> Result<Buffer, ReedSolomonError> {
        let mut msg = Buffer::from_slice(msg, msg.len() - self.ecc_len);

        assert!(msg.len() < 256);

        let erase_pos = if let Some(erase_pos) = erase_pos {
            for e_pos in erase_pos {
                msg[*e_pos as usize] = 0;
            }
            erase_pos
        } else {
            &[]
        };

        if erase_pos.len() > self.ecc_len {
            return Err(ReedSolomonError::TooManyErrors);
        }

        let synd = self.calc_syndromes(&msg);

        // No errors
        if synd.iter().max() == Some(&0) {
            return Ok(msg);
        }

        let fsynd = self.forney_syndromes(&synd, &erase_pos, msg.len());
        let err_loc = self.find_error_locator(&fsynd, None, erase_pos.len())?;
        let mut err_pos = self.find_errors(&err_loc.reverse(), msg.len())?;

        // Append erase_pos to err_pos
        for x in erase_pos.iter() {
            err_pos.push(*x);
        }

        let msg_out = self.correct_errata(&msg, &synd, &err_pos);

        // Check output message correctness
        if self.is_corrupted(&msg_out) {
            Err(ReedSolomonError::TooManyErrors)
        } else {
            Ok(Buffer::from_polynom(msg_out, msg.len() - self.ecc_len))
        }
    }

    fn calc_syndromes(&self, msg: &[u8]) -> Polynom {
        // index 0 is a pad for mathematical precision
        let mut synd = Polynom::with_length(self.ecc_len + 1);
        for i in 0..self.ecc_len {
            uncheck_mut!(synd[i + 1]) = msg.eval(gf::pow(2, i as i32))
        }

        synd
    }

    fn is_corrupted(&self, msg: &[u8]) -> bool {
        self.calc_syndromes(msg).iter().max() != Some(&0)
    }

    fn find_errata_locator(&self, e_pos: &[u8]) -> Polynom {
        let mut e_loc = polynom![1];

        let add_lhs = [1];
        let mut add_rhs = [0, 0];
        for i in e_pos.iter() {
            add_rhs[0] = gf::pow(2, *i as i32);
            e_loc = e_loc.mul(&add_lhs.add(&add_rhs));
        }

        e_loc
    }

    fn find_error_evaluator(&self, synd: &[u8], err_loc: &[u8], syms: usize) -> Polynom {
        let mut divisor = Polynom::with_length(syms + 2);
        divisor[0] = 1;

        let (_, remainder) = (synd.mul(err_loc)).div(&divisor);
        remainder
    }

    /// Forney algorithm, computes the values (error magnitude) to correct the input message.
    #[allow(non_snake_case)]
    fn correct_errata(&self, msg: &[u8], synd: &[u8], err_pos: &[u8]) -> Polynom {
        // convert the positions to coefficients degrees
        let mut coef_pos = Polynom::with_length(err_pos.len());
        for (i, x) in err_pos.iter().enumerate() {
            coef_pos[i] = msg.len() as u8 - 1 - x;
        }

        let err_loc = self.find_errata_locator(&coef_pos);
        let synd = Polynom::copy_from_slice(synd);
        let err_eval = self.find_error_evaluator(&synd.reverse(), &err_loc, err_loc.len() - 1)
                    .reverse();

        let mut X = Polynom::new();

        for px in coef_pos.iter() {
            let l = (255 - px) as i32;
            X.push(gf::pow(2, -l))
        }

        let mut E = Polynom::with_length(msg.len());

        let err_eval_rev = err_eval.reverse();
        for (i, Xi) in X.iter().enumerate() {
            let Xi_inv = gf::inverse(*Xi);

            let mut err_loc_prime_tmp = Polynom::new();
            for j in 0..X.len() {
                if j != i {
                    err_loc_prime_tmp.push(gf::sub(1, gf::mul(Xi_inv, X[j])));
                }
            }

            let mut err_loc_prime = 1;
            for coef in err_loc_prime_tmp.iter() {
                err_loc_prime = gf::mul(err_loc_prime, *coef);
            }

            let y = err_eval_rev.eval(Xi_inv);
            let y = gf::mul(gf::pow(*Xi, 1), y);

            let magnitude = gf::div(y, err_loc_prime);

            let E_index = uncheck!(err_pos[i]) as usize;
            uncheck_mut!(E[E_index]) = magnitude;
        }

        msg.add(&E)
    }

    #[allow(non_snake_case)]
    fn find_error_locator(&self,
                          synd: &[u8],
                          erase_loc: Option<&[u8]>,
                          erase_count: usize)
                          -> Result<Polynom, ReedSolomonError> {
        let (mut err_loc, mut old_loc) = if let Some(erase_loc) = erase_loc {
            (Polynom::from(erase_loc), Polynom::from(erase_loc))
        } else {
            (polynom![1], polynom![1])
        };

        let synd_shift = if synd.len() > self.ecc_len {
            synd.len() - self.ecc_len
        } else {
            0
        };

        for i in 0..(self.ecc_len - erase_count) {
            let K = if erase_loc.is_some() {
                erase_count + i + synd_shift
            } else {
                i + synd_shift
            };

            let mut delta = uncheck!(synd[K]);
            for j in 1..err_loc.len() {
                let d_index = err_loc.len() - j - 1;
                delta ^= gf::mul(err_loc[d_index], uncheck!(synd[K - j]));
            }

            old_loc.push(0);

            if delta != 0 {
                if old_loc.len() > err_loc.len() {
                    let new_loc = old_loc.scale(delta);
                    old_loc = err_loc.scale(gf::inverse(delta));
                    err_loc = new_loc;
                }

                err_loc = err_loc.add(&old_loc.scale(delta));
            }
        }

        let mut shift = 0;
        while err_loc.len() - shift > 0 && err_loc[shift] == 0 {
            shift += 1;
        }

        err_loc.reverse();
        err_loc.length = err_loc.len() - shift;
        err_loc.reverse();

        let errs = err_loc.len() - 1;

        let errs = if erase_count > errs {
            erase_count
        } else {
            (errs - erase_count) * 2 + erase_count
        };

        if errs > self.ecc_len {
            Err(ReedSolomonError::TooManyErrors)
        } else {
            Ok(err_loc)
        }
    }

    fn find_errors(&self, err_loc: &[u8], msg_len: usize) -> Result<Polynom, ReedSolomonError> {
        let errs = err_loc.len() - 1;
        let mut err_pos = polynom![];

        for i in 0..msg_len {
            if err_loc.eval(gf::pow(2, i as i32)) == 0 {
                let x = msg_len as u8 - 1 - i as u8;
                err_pos.push(x);
            }
        }

        if err_pos.len() != errs {
            Err(ReedSolomonError::TooManyErrors)
        } else {
            Ok(err_pos)
        }
    }

    fn forney_syndromes(&self, synd: &[u8], pos: &[u8], msg_len: usize) -> Polynom {
        let mut erase_pos_rev = Polynom::with_length(pos.len());
        for (i, x) in pos.iter().enumerate() {
            erase_pos_rev[i] = msg_len as u8 - 1 - x;
        }

        let mut fsynd = Polynom::copy_from_slice(&synd[1..]);

        for i in 0..pos.len() {
            let x = gf::pow(2, erase_pos_rev[i] as i32);
            for j in 0..(fsynd.len() - 1) {
                fsynd[j] = gf::mul(fsynd[j], x) ^ fsynd[j + 1];
            }
        }

        fsynd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::Encoder;

    #[test]
    fn calc_syndromes() {
        let px = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut encoded = Encoder::new(8).encode(&px[..]);

        assert_eq!([0; 9], *Decoder::new(8).calc_syndromes(&encoded));

        encoded[5] = 1;

        assert_eq!([0, 7, 162, 172, 245, 176, 71, 58, 180],
                   *Decoder::new(8).calc_syndromes(&encoded));
    }

    #[test]
    fn is_corrupted() {
        let px = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut encoded = Encoder::new(8).encode(&px[..]);

        assert_eq!(false, Decoder::new(8).is_corrupted(&encoded));

        encoded[5] = 1;

        assert_eq!(true, Decoder::new(8).is_corrupted(&encoded));
    }

    #[test]
    fn find_errata_locator() {
        let e_pos = [19, 18, 17, 14, 15, 16];
        assert_eq!([134, 207, 111, 227, 24, 150, 1],
                   *Decoder::new(6).find_errata_locator(&e_pos[..]));
    }

    #[test]
    fn find_error_evaluator() {
        let synd = [232, 103, 78, 56, 109, 59, 242, 42, 64, 0];
        let err_loc = [134, 207, 111, 227, 24, 150, 1];

        assert_eq!([148, 151, 175, 126, 68, 64, 0],
                   *Decoder::new(6).find_error_evaluator(&synd, &err_loc, 6));
    }

    #[test]
    fn correct_errata() {
        let msg = [0, 0, 0, 2, 2, 2, 119, 111, 114, 108, 100, 145, 124, 96, 105, 94, 31,
                           179, 149, 163];
        let synd = [0, 64, 42, 242, 59, 109, 56, 78, 103, 232];
        let err_pos = [0, 1, 2, 5, 4, 3];
        let result = [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 145, 124, 96, 105, 94,
                      31, 179, 149, 163];

        assert_eq!(result,
                   *Decoder::new(err_pos.len()).correct_errata(&msg, &synd, &err_pos));
    }

    #[test]
    fn find_error_locator() {
        let synd = [79, 25, 0, 160, 198, 122, 192, 169, 232];
        let nsym = 9;
        let erase_loc = None;
        let erase_count = 3;

        let result = [193, 144, 121, 1];

        let error_loc = Decoder::new(nsym).find_error_locator(&synd, erase_loc, erase_count);

        assert!(error_loc.is_ok());
        assert_eq!(result, *error_loc.unwrap());
    }

    #[test]
    fn find_errors() {
        let err_loc = [1, 121, 144, 193];
        let msg_len = 20;
        let result = [5, 4, 3];

        let err_pos = Decoder::new(6).find_errors(&err_loc, msg_len);

        assert!(err_pos.is_ok());
        assert_eq!(result, *err_pos.unwrap());

        let err_loc = [1, 134, 181];
        let msg_len = 12;

        let err_pos = Decoder::new(6).find_errors(&err_loc, msg_len);

        assert!(err_pos.is_err());
    }

    #[test]
    fn forney_syndromes() {
        let synd = [0, 64, 42, 242, 59, 109, 56, 78, 103, 232];
        let pos = [0, 1, 2];
        let nmess = 20;

        let result = [79, 25, 0, 160, 198, 122, 192, 169, 232];
        assert_eq!(result,
                   *Decoder::new(6).forney_syndromes(&synd, &pos, nmess));
    }

    #[test]
    fn decode() {
        let msg = [0, 2, 2, 2, 2, 2, 119, 111, 114, 108, 100, 145, 124, 96, 105, 94, 31, 179, 149,
                   163];
        let ecc = 9;
        let erase_pos = [0, 1, 2];

        let result = [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 145, 124, 96, 105, 94,
                      31, 179, 149, 163];

        let decoder = Decoder::new(ecc);
        let decoded = decoder.decode(&msg[..], Some(&erase_pos)).unwrap();

        assert_eq!(result, **decoded);
    }
}
