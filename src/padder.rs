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


pub struct Padder {
    block_size: usize,
    length_size: LengthSize,
}

impl Padder {
    pub fn new(block_size: usize, length_size: LengthSize) -> Padder {
        Padder{block_size, length_size}
    }

    fn pad_with_len(&self, total_length: ByteCounter, data: &mut Vec<u8>) {
        let mut len_bytes = len_in_bits_encoded_as_bytes(total_length, self.length_size);
        data.resize(self.block_size-len_bytes.len(), 0);
        data.append(&mut len_bytes);
    }

    pub fn is_full_block(&self, size: usize) -> bool {
        size == self.block_size
    }

    // Is there room for padding+length without having to make an extra block?
    pub fn is_room(&self, size: usize) -> bool {
        size+self.length_size.byte_size() < self.block_size
    }

    // total_length must account for all previous blocks as well as the supplied data block.
    pub fn single_pad(&self, data: &[u8], total_length: ByteCounter) -> Vec<u8> {

        let mut result = data.to_vec();
        result.push(0x80);
        self.pad_with_len(total_length, &mut result);

        result
    }

    pub fn double_pad_1st_part(&self, data: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        result.push(0x80);
        result.resize(self.block_size, 0);

        result
    }

    // total_length must account for all previous blocks as well as the data block supplied to the 1st part.
    pub fn double_pad_2nd_part(&self, total_length: ByteCounter) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        self.pad_with_len(total_length, &mut result);

        result
    }
}

pub struct ShaPaddedStream<I> where I: IntoIterator<Item = InputItemType> {
    block_iter: I::IntoIter,
    length_in_bytes: ByteCounter,
    padder: Padder,
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
        let padder = Padder::new(block_length, length_size);
        ShaPaddedStream{block_iter: blockstream.into_iter(), padder, length_in_bytes:0, padding_started: false, done: false}
    }

}

impl<'a, I> Iterator for ShaPaddedStream<I> where I: IntoIterator<Item = InputItemType> {
    type Item = OutputItemType;

    fn next(&mut self) -> Option<OutputItemType> {
        match self.block_iter.next() {
            Some(block) => {
                self.length_in_bytes += u128::try_from(block.len()).unwrap(); //usize is platformdependant

                if self.padder.is_full_block(block.len()) {
                    return Some(block);
                } else {
                    let result;

                    if self.padder.is_room(block.len()) {
                        result = self.padder.single_pad(&block, self.length_in_bytes);
                        self.done = true;
                    } else {
                        result = self.padder.double_pad_1st_part(&block);
                        self.padding_started = true;
                    }

                    return Some(result);
                }
            }
            None => {
                if self.done {
                    return None
                } else {
                    self.done = true;
                    if self.padding_started {
                        return Some(self.padder.double_pad_2nd_part(self.length_in_bytes));
                    } else {
                        return Some(self.padder.single_pad(&[], self.length_in_bytes))
                    }
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
