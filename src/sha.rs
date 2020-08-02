use crate::block_splitter;
use crate::padder::{ShaPaddedStream, LengthSize};
use crate::sha256::{sha256_block, u32_to_u8, H0};

pub fn sha<'a, I>(msg: I) -> Vec<u8> where I: IntoIterator<Item=u8>
{
    let blocksize = 64;

    let block_stream = block_splitter::BlockStream::new(blocksize, msg.into_iter());
    let padded_stream = ShaPaddedStream::new(block_stream, blocksize, LengthSize::Len64).collect::<Vec<_>>();

    let mut h = H0;
    for block in padded_stream {
        h = sha256_block(h, block);
    }

    return u32_to_u8(h.to_vec());
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate hex;

    #[test]
    fn test_abc_hash() {
        let result = sha("abc".bytes());
        assert_eq!(hex::encode(result), "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD".to_lowercase());
    }

    #[test]
    fn test_twoblock_hash() {
        let result = sha("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".bytes());
        assert_eq!(hex::encode(result), "248D6A61D20638B8E5C026930C3E6039A33CE45964FF2167F6ECEDD419DB06C1".to_lowercase());
    }

}
