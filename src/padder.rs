use std::convert::TryFrom;

type ByteCounter = u128;
type InputItemType = Vec<u8>;
type OutputItemType = Vec<u8>;

#[derive(Copy, Clone)]
pub enum LengthSize { Len64, Len128 }

impl LengthSize {
    fn byte_size(&self) -> usize {
        match *self {
            LengthSize::Len64 => 8,
            LengthSize::Len128 => 16,
        }
    }
}

pub struct ShaPaddedStream<I> where I: IntoIterator<Item = InputItemType> {
    block_iter: I::IntoIter,
    block_length: usize,
    length_in_bytes: ByteCounter,
    length_size: LengthSize,
    padding_started: bool,
    done: bool,
}

fn len_in_bits_encoded_as_bytes(len: ByteCounter, length_size: LengthSize) -> Vec<u8> {
    let bit_len = len * 8;
    match length_size {
        LengthSize::Len128 => return bit_len.to_be_bytes().to_vec(),
        LengthSize::Len64 => {
            if bit_len > u64::MAX as u128 {
                panic!("Message is larger than the format allows")
            } else {
                return (bit_len as u64).to_be_bytes().to_vec()
            }
        },
    }
}

impl<'a, I> ShaPaddedStream<I> where I: IntoIterator<Item = InputItemType> {
    pub fn new(blockstream: I, block_length: usize, length_size: LengthSize) -> ShaPaddedStream<I> {
        ShaPaddedStream{block_iter: blockstream.into_iter(), block_length, length_in_bytes:0, length_size, padding_started: false, done: false}
    }

    fn pad_with_len(&mut self, buffer: &mut OutputItemType) {
        let mut len_bytes = len_in_bits_encoded_as_bytes(self.length_in_bytes, self.length_size);
        buffer.resize(self.block_length-len_bytes.len(), 0);
        buffer.append(&mut len_bytes);
    }
}

impl<'a, I> Iterator for ShaPaddedStream<I> where I: IntoIterator<Item = InputItemType> {
    type Item = OutputItemType;

    fn next(&mut self) -> Option<OutputItemType> {
        match self.block_iter.next() {
            Some(block) => {
                self.length_in_bytes += u128::try_from(block.len()).unwrap(); //usize is platformdependant

                if block.len() < self.block_length {
                    let mut result = block.to_vec();
                    result.push(0x80); //start of padding marker

                    if result.len()+self.length_size.byte_size() > self.block_length {
                        // not enough room for the entire size, so just pad out, and complete it
                        // on the next call to next()
                        result.resize(self.block_length, 0);
                        self.padding_started = true;
                    } else {
                        self.pad_with_len(&mut result);
                        self.done = true;
                    }
                    return Some(result);
                }

                return Some(block);
            }
            None => {
                if self.done {
                    return None
                } else {
                    self.done = true;

                    let mut result = Vec::new();
                    if !self.padding_started {
                        result.push(0x80);
                    }
                    self.pad_with_len(&mut result);
                    return Some(result);
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    use crate::block_splitter;

    extern crate rstest;
    use rstest::rstest;

    extern crate hex;

    #[rstest(input, expected,
      case::empty( 0u8..0u8,
          vec!["80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"]),

      case::one(   1u8..2u8,
          vec!["01800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008"]),

      case::a_few( 1u8..16u8,
          vec!["0102030405060708090a0b0c0d0e0f80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000078"]),

      case::one_block( 0u8..0x37u8,
          vec!["000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435368000000000000001b8"]),

      case::one_block_and_one( 0u8..0x38u8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30313233343536378000000000000000",
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c0"
            ]
        ),

      case::one_less_than_full_first_block( 0u8..0x3fu8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e80",
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001f8"
            ]
        ),

      case::first_block_just_full( 0u8..0x40u8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200"
            ]
        ),

      case::first_block_and_one( 0u8..0x41u8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "40800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000208"
            ]
        ),

      case::just_two_blocks( 0u8..0x77u8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475768000000000000003b8"
            ]
        ),

      case::just_two_blocks_and_one( 0u8..0x78u8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f70717273747576778000000000000000",
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c0"
            ]
        ),


      case::one_less_than_two_full_blocks( 0u8..0x7fu8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e80",
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003f8"
            ]
        ),

      case::two_full_blocks( 0u8..0x80u8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f",
            "80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400"
            ]
        ),

      case::two_full_blocks_and_one( 0u8..0x81u8,
          vec![
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f",
            "80800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000408"
            ]
        ),
    )]
    fn test_sha_32_padding_stream(input: Range<u8>, expected: Vec<&str>) {
        let blocksize = 64;

        let blockstream = block_splitter::BlockStream::new(blocksize, input.into_iter());
        let result = ShaPaddedStream::new(blockstream, blocksize, LengthSize::Len64).collect::<Vec<_>>();

        let hexstr = result.iter().map(|v| hex::encode(v)).collect::<Vec<_>>();
        assert_eq!(hexstr, expected);
    }

}
