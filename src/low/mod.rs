mod macros;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::{
    curve25519_x25519::curve25519_x25519, curve25519_x25519base::curve25519_x25519base,
};

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::{
    curve25519_x25519::curve25519_x25519, curve25519_x25519base::curve25519_x25519base,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc7748_1() {
        let scalar = b"\xa5\x46\xe3\x6b\xf0\x52\x7c\x9d\x3b\x16\x15\x4b\x82\x46\x5e\xdd\x62\x14\x4c\x0a\xc1\xfc\x5a\x18\x50\x6a\x22\x44\xba\x44\x9a\xc4";
        let point = b"\xe6\xdb\x68\x67\x58\x30\x30\xdb\x35\x94\xc1\xa4\x24\xb1\x5f\x7c\x72\x66\x24\xec\x26\xb3\x35\x3b\x10\xa9\x03\xa6\xd0\xab\x1c\x4c";
        let mut res = [0u8; 32];
        curve25519_x25519(&mut res, scalar, point);
        assert_eq!(
            &res[..],
            b"\xc3\xda\x55\x37\x9d\xe9\xc6\x90\x8e\x94\xea\x4d\xf2\x8d\x08\x4f\x32\xec\xcf\x03\x49\x1c\x71\xf7\x54\xb4\x07\x55\x77\xa2\x85\x52"
        );
    }

    #[test]
    fn rfc7748_2() {
        let scalar = b"\x4b\x66\xe9\xd4\xd1\xb4\x67\x3c\x5a\xd2\x26\x91\x95\x7d\x6a\xf5\xc1\x1b\x64\x21\xe0\xea\x01\xd4\x2c\xa4\x16\x9e\x79\x18\xba\x0d";
        let point = b"\xe5\x21\x0f\x12\x78\x68\x11\xd3\xf4\xb7\x95\x9d\x05\x38\xae\x2c\x31\xdb\xe7\x10\x6f\xc0\x3c\x3e\xfc\x4c\xd5\x49\xc7\x15\xa4\x93";
        let mut res = [0u8; 32];
        curve25519_x25519(&mut res, scalar, point);
        assert_eq!(
            &res[..],
            b"\x95\xcb\xde\x94\x76\xe8\x90\x7d\x7a\xad\xe4\x5c\xb4\xb8\x73\xf8\x8b\x59\x5a\x68\x79\x9f\xa1\x52\xe6\xf8\xf7\x64\x7a\xac\x79\x57"
        );
    }

    #[test]
    fn rfc7748_3() {
        let mut k = [0u8; 32];
        k[0] = 9;
        let mut res = [0u8; 32];

        // After one iteration: 422c8e7a6227d7bca1350b3e2bb7279f7897b87bb6854b783c60e80311ae3079
        curve25519_x25519base(&mut res, &k);
        assert_eq!(
            &res[..],
            b"\x42\x2c\x8e\x7a\x62\x27\xd7\xbc\xa1\x35\x0b\x3e\x2b\xb7\x27\x9f\x78\x97\xb8\x7b\xb6\x85\x4b\x78\x3c\x60\xe8\x03\x11\xae\x30\x79",
        );

        // After 1,000 iterations: 684cf59ba83309552800ef566f2f4d3c1c3887c49360e3875f2eb94d99532c51
        let mut u = [0u8; 32];
        u.copy_from_slice(&k);
        k.copy_from_slice(&res);

        for _ in 1..1000 {
            curve25519_x25519(&mut res, &k, &u);
            u.copy_from_slice(&k);
            k.copy_from_slice(&res);
        }

        assert_eq!(
            &k[..],
            b"\x68\x4c\xf5\x9b\xa8\x33\x09\x55\x28\x00\xef\x56\x6f\x2f\x4d\x3c\x1c\x38\x87\xc4\x93\x60\xe3\x87\x5f\x2e\xb9\x4d\x99\x53\x2c\x51"
        );

        if option_env!("SLOW_TESTS").is_some() {
            // After 1,000,000 iterations: 7c3911e0ab2586fd864497297e575e6f3bc601c0883c30df5f4dd2d24f665424
            for _ in 1000..1000_000 {
                curve25519_x25519(&mut res, &k, &u);
                u.copy_from_slice(&k);
                k.copy_from_slice(&res);
            }

            assert_eq!(
                &k[..],
                b"\x7c\x39\x11\xe0\xab\x25\x86\xfd\x86\x44\x97\x29\x7e\x57\x5e\x6f\x3b\xc6\x01\xc0\x88\x3c\x30\xdf\x5f\x4d\xd2\xd2\x4f\x66\x54\x24",
            );
        }
    }

    #[test]
    fn base_mul() {
        let scalar = [1u8; 32];
        let mut res = [0u8; 32];
        curve25519_x25519base(&mut res, &scalar);
        // generated manually with cryptography.io
        assert_eq!(
            &res[..],
            b"\xa4\xe0\x92\x92\xb6\x51\xc2\x78\xb9\x77\x2c\x56\x9f\x5f\xa9\xbb\x13\xd9\x06\xb4\x6a\xb6\x8c\x9d\xf9\xdc\x2b\x44\x09\xf8\xa2\x09",
        );
    }
}
