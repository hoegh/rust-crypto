use std::convert::TryInto;

pub const H0:[u32;8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
const K:[u32;64] = [
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
];

fn message_schedule(m: Vec<u8>) -> [u32;64] {
    let mut w: [u32;64] = [0;64];

    m.chunks(4)
        .enumerate()
        .for_each(
            |(i, n)| w[i]=u32::from_be_bytes(n.try_into().unwrap())
        );

    for t in 16..64 {
        w[t] = sigma!(w[t-2], 17, 19, 10)
            .wrapping_add(w[t-7])
            .wrapping_add(sigma!(w[t-15], 7, 18, 3))
            .wrapping_add(w[t-16]);
    }

    return w;
}

pub fn sha256_block(hash: [u32;8], m: Vec<u8>) -> [u32;8] {
    let w = message_schedule(m);
    let mut a = hash[0];
    let mut b = hash[1];
    let mut c = hash[2];
    let mut d = hash[3];
    let mut e = hash[4];
    let mut f = hash[5];
    let mut g = hash[6];
    let mut h = hash[7];

    for t in 0..64 {
        let t1 = h
            .wrapping_add( Sigma!(e, 6, 11, 25))
            .wrapping_add(ch!(e, f, g))
            .wrapping_add(K[t])
            .wrapping_add(w[t]);
        let t2 = Sigma!(a, 2, 13, 22).wrapping_add(maj!(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    return [
        a.wrapping_add(hash[0]),
        b.wrapping_add(hash[1]),
        c.wrapping_add(hash[2]),
        d.wrapping_add(hash[3]),
        e.wrapping_add(hash[4]),
        f.wrapping_add(hash[5]),
        g.wrapping_add(hash[6]),
        h.wrapping_add(hash[7])
        ];
}

pub fn u32_to_u8(wa: Vec<u32>)->Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    for w in wa {
        result.extend_from_slice(&w.to_be_bytes());
    }

    return result;
}



#[cfg(test)]
mod tests {
    use super::*;

    extern crate hex;

    #[test]
    fn test_abc_hash() {
        let m = hex::decode("61626380000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018").unwrap();
        let mut result = sha256_block(H0, m);
        let result_bytes = u32_to_u8(result.to_vec());
        assert_eq!(hex::encode(result_bytes), "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD".to_lowercase());
    }

}
