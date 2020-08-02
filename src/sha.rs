use crate::block_splitter;
use crate::padder::{ShaPaddedStream, LengthSize};
use crate::sha256::{sha256_block, u32_to_u8, H0 as SHA256_H0};
use crate::sha512::{sha512_block, u64_to_u8, H0 as SHA512_H0};

pub struct ShaParams<T> {
    block_size: usize,
    length_size: LengthSize,
    h0: [T;8],
    sha_func: fn([T;8], Vec<u8>) -> [T;8],
    convert_func: fn(Vec<T>) -> Vec<u8>,
}

pub const SHA256: ShaParams<u32> = ShaParams {
    block_size: 64,
    length_size: LengthSize::Len64,
    h0: SHA256_H0,
    sha_func: sha256_block,
    convert_func: u32_to_u8
};

pub const SHA512: ShaParams<u64> = ShaParams {
    block_size: 128,
    length_size: LengthSize::Len128,
    h0: SHA512_H0,
    sha_func: sha512_block,
    convert_func: u64_to_u8
};

pub fn sha<'a, I, T>(params: ShaParams<T>, msg: I) -> Vec<u8> where I: IntoIterator<Item=u8>, T: std::clone::Clone
{
    let block_stream = block_splitter::BlockStream::new(params.block_size, msg.into_iter());
    let padded_stream = ShaPaddedStream::new(block_stream, params.block_size, params.length_size);

    let mut h = params.h0;
    for block in padded_stream {
        h = (params.sha_func)(h, block);
    }

    return (params.convert_func)(h.to_vec());
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate hex;

    #[test]
    fn test_sha256_abc_hash() {
        let result = sha(SHA256, "abc".bytes());
        assert_eq!(hex::encode(result), "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD".to_lowercase());
    }

    #[test]
    fn test_sha256_twoblock_hash() {
        let result = sha(SHA256, "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".bytes());
        assert_eq!(hex::encode(result), "248D6A61D20638B8E5C026930C3E6039A33CE45964FF2167F6ECEDD419DB06C1".to_lowercase());
    }

    #[test]
    fn test_sha512_abc_hash() {
        let result = sha(SHA512, "abc".bytes());
        assert_eq!(hex::encode(result), "DDAF35A193617ABACC417349AE20413112E6FA4E89A97EA20A9EEEE64B55D39A2192992A274FC1A836BA3C23A3FEEBBD454D4423643CE80E2A9AC94FA54CA49F".to_lowercase());
    }

    #[test]
    fn test_sha512_twoblock_hash() {
        let result = sha(SHA512, "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu".bytes());
        assert_eq!(hex::encode(result), "8E959B75DAE313DA8CF4F72814FC143F8F7779C6EB9F7FA17299AEADB6889018501D289E4900F7E4331B99DEC4B5433AC7D329EEB6DD26545E96E55B874BE909".to_lowercase());
    }

}
