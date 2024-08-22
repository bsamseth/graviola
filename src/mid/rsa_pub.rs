// Written for Graviola by Joe Birr-Pixton, 2024.
// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use crate::error::Error;
use crate::low;

#[derive(Clone, Debug)]
pub(crate) struct RsaPublicKey {
    pub(crate) n: RsaPosInt,
    e: u32,

    montifier: RsaPosInt,
    one: RsaPosInt,
    n0: u64,
}

impl RsaPublicKey {
    pub(crate) fn new(n: RsaPosInt, e: u32) -> Result<Self, Error> {
        let n_len = n.len_bytes();
        if n.is_even()
            || !(MIN_PUBLIC_MODULUS_BYTES..=MAX_PUBLIC_MODULUS_BYTES).contains(&n_len)
            || e == 0
        {
            return Err(Error::OutOfRange);
        }

        // determine M^2 mod n
        let montifier = n.montifier();

        // and its inverse such that n * n0 == -1 (mod 2^64)
        let n0 = n.mont_neg_inverse();

        // and just M
        let one = RsaPosInt::one().mont_mul(&montifier, &n, n0);

        Ok(Self {
            n,
            e,
            montifier,
            one,
            n0,
        })
    }

    pub(crate) fn modulus_len_bytes(&self) -> usize {
        self.n.len_bytes()
    }

    /// m = c ** e mod n
    pub(crate) fn public_op(&self, c: &RsaPosInt) -> Result<RsaPosInt, Error> {
        if !c.less_than(&self.n) {
            return Err(Error::OutOfRange);
        }

        // bring c into montgomery domain, c_mont = c * M^2 * M^-1 mod n
        let c_mont = c.to_montgomery(&self.montifier, &self.n);

        // accumulator is 1 * 1 in montgomery domain, ie, just M
        let mut accum = self.one.clone();

        let mut first = true;
        for bit in (0..self.e.ilog2() + 1).rev() {
            let tmp = if first {
                // avoid pointless squaring of multiplicative identity
                first = false;
                accum
            } else {
                accum.mont_sqr(&self.n, self.n0)
            };

            let mask = 1 << bit;
            if self.e & mask == mask {
                accum = tmp.mont_mul(&c_mont, &self.n, self.n0);
            } else {
                accum = tmp;
            }
        }

        // drop accumulator out of montgomery domain
        Ok(accum.from_montgomery(&self.n))
    }
}

const MAX_PUBLIC_MODULUS_BITS: usize = 8192;
const MAX_PUBLIC_MODULUS_WORDS: usize = MAX_PUBLIC_MODULUS_BITS / 64;
pub(crate) const MAX_PUBLIC_MODULUS_BYTES: usize = MAX_PUBLIC_MODULUS_BITS / 8;

const MIN_PUBLIC_MODULUS_BITS: usize = 2048;
const MIN_PUBLIC_MODULUS_BYTES: usize = MIN_PUBLIC_MODULUS_BITS / 8;

type RsaPosInt = low::PosInt<MAX_PUBLIC_MODULUS_WORDS>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let n = RsaPosInt::from_bytes(b"\xe4\x46\x29\x68\xe3\xe2\x9c\xe7\x3b\xe8\xac\xda\xf9\xd5\x92\xbe\x99\x04\x36\x3a\xef\x33\x99\xf7\x93\xb9\x17\x13\x42\x9c\xea\xf9\x63\xa1\xe5\xc6\xbb\x57\x71\x4c\xc1\x46\x01\xec\xac\x5a\xe5\xb8\x95\x43\xaa\xfa\x68\x3d\x50\x73\x87\xfc\x83\x04\x66\x1f\xab\x1e\x0c\x6e\xf0\x32\x50\x63\x21\xc6\x74\xec\xe4\xf6\x7a\xb2\x94\xbe\xae\x81\x66\x3e\x1a\xa6\x98\xcd\x5b\x78\x2c\x7b\xf4\xdf\x39\x76\xf1\x5e\x88\xda\xa2\xe0\xe8\x2e\xb5\x83\xdb\x1b\x56\xe4\x6b\x6f\x4e\x3c\xde\x9f\x00\x7e\x3b\x8f\x8f\x5c\xb8\x55\x04\x22\xea\x1f\x6d\x92\xe1\x08\x76\x2a\x68\xc5\x35\xd2\x37\x9a\x54\xdc\xf7\x4f\x19\x38\xdb\x77\x02\xd9\xf9\x72\x4d\x7f\x98\xa5\xe3\x7c\xef\x06\xc7\xb0\x3f\x58\xbc\x9d\x38\x72\x8a\xac\x18\x03\xb9\xee\x60\xe7\x6e\x18\xf6\x90\x87\xb3\x8a\x5f\xbb\x95\xd0\x99\x09\x5b\x2c\xda\x4b\xd7\x88\xaa\x2a\x05\x07\x38\xae\xf6\xa1\x6e\x93\x00\x1f\xc3\x6b\xb4\xdc\x6b\xc1\xc6\x06\x1e\x34\x9c\x5b\x2b\xd6\x50\x5d\x64\xd9\x05\xdb\x95\xa0\xe1\x2c\xb3\xb1\x5b\xa4\x90\xa2\xa7\xcc\xbf\x10\xaf\x12\xe3\x16\xb3\xde\xc5\x4f\xb1\xb6\x63\x68\xd8\xd9\xb1").unwrap();
        let c = RsaPosInt::from_bytes(b"\x00\x0b\x36\xb5\xc6\xd9\x32\xd0\x18\xa6\x31\x99\x82\xf6\xba\x83\xd5\x1b\xb6\xdb\x84\x99\x87\xc0\xe9\x8f\x06\x63\xac\x8d\xe4\x43\xb0\x45\xd3\x01\x3e\x03\xba\xed\xd0\xa9\xc6\x49\x08\x63\x22\x29\x0f\x1f\xf3\x25\xef\xfe\x65\xff\x27\xf2\x5d\xc6\xe7\x79\xe9\x5f\xd2\xf5\x09\x0c\x28\xfe\xe5\x6c\x75\x24\x0a\x79\xe4\xf6\x9e\x2b\x5b\x52\x71\xb6\x22\xd8\x08\x97\xea\xbd\x4b\x06\x53\xa6\x2e\xb9\x26\x91\x0f\xc7\x34\xa4\x5d\x3b\x9d\x23\xc0\x10\xf8\x82\xa7\xbb\x8c\x50\x35\x7d\x44\x9d\x14\x00\xcf\x5a\xe0\x92\xeb\x83\x60\x9a\x48\xbc\xac\xe0\x20\xd7\x44\xc9\xe7\xf7\x66\x25\x04\x0e\xa9\x20\x9c\xb6\x23\x02\x8f\x2b\xa3\x86\xfa\x23\x4e\xdd\xe9\xf8\xc8\xa4\x63\x65\x4c\x9d\x52\x24\x4a\x0d\x0a\xd6\x2d\x94\x95\x64\x45\xaa\xf9\xf5\x26\x8b\xf7\x21\xf7\x6a\xf9\x19\x46\xbc\x2e\xeb\x2a\xaf\x0f\x31\x2f\x27\x86\x4e\xd4\x2e\xf7\xbc\x0f\x14\xce\x75\xef\x93\xad\x3a\x84\x3a\xb3\x29\x6f\xe9\xd7\x33\xd8\x6c\xbe\x20\x11\xf3\x92\x3c\x16\x78\x0b\xc4\x79\xaa\x8d\xeb\xb1\xd1\xe2\xda\xf3\xd7\x43\x92\x72\x8c\x81\x52\x3d\xf1\xc9\x7e\x7c\xfd\x0e\xb2\x02\x84\x51").unwrap();

        let k = RsaPublicKey::new(n, 0x10001).unwrap();
        let m = k.public_op(&c).unwrap();
        println!("m = {:016x?}", m);

        let mut mb = [0; 256];
        let mb = m.to_bytes(&mut mb).unwrap();
        println!("m = {:02x?}", mb);
    }
}
