mod iters {
    use std::mem;

    use super::conv::u32_from_vec_slice;

    struct BlockParams{
        size: usize,
        reserve: usize,
    }

    /// SHA message stream padder
    ///
    /// SHA hashes are calculated over fixed length block with the length appended. Thus the message
    /// stream must be padded.
    struct ShaPadder<I> where I: IntoIterator<Item = u8> {
        block_params: BlockParams,
        has_message: bool,
        message_count: usize,
        padding_left: usize,
        bit_size_as_bytes: [u8;8], //size is measure in bits, but these are the bytes that make up the number
        message_iter: I::IntoIter,
    }

    impl<I> ShaPadder<I> where I: IntoIterator<Item = u8> {
        /// Given the block parameters, take a message stream of u8 data and implement an Iterator
        /// that emits a padded stream of u8 data.
        pub fn new(params: BlockParams, message: I) -> ShaPadder<I> {
            ShaPadder{block_params: params, message_count: 0, has_message: true, message_iter: message.into_iter(),
                bit_size_as_bytes: [0u8;8], padding_left: 0}
        }
    }

    impl<I> Iterator for ShaPadder<I> where I: IntoIterator<Item = u8> {
        type Item = u8;

        fn next(&mut self) -> Option<u8> {
            if self.has_message {
                // stage one, pass on message
                match self.message_iter.next() {
                    Some(b) => {
                        self.message_count += 1;
                        Some(b)
                    }
                    None => {
                        //out of message - calculate padding length and size in bits
                        self.has_message = false; //go to stage 2
                        self.bit_size_as_bytes = ((self.message_count*8) as u64).to_le_bytes(); //the size is measure in bits, but counted in bytes

                        self.padding_left = self.block_params.size - (self.message_count % self.block_params.size) - 1;
                        if self.padding_left < self.block_params.reserve {
                            // if there isn't enough space for the reserve, add another blocksize worth of padding
                            self.padding_left += self.block_params.size;
                        }

                        //emit start-of-padding marker
                        Some(0x80u8)
                    }
                }
            } else {
                // stage two, emit padding and bit size counter
                if self.padding_left == 0 {
                    None
                } else {
                    self.padding_left -= 1;
                    if self.padding_left >= mem::size_of::<u64>() {
                        // still padding left to be emitted before the bit size counter
                        Some(0)
                    } else {
                        // emit a byte from the bit size counter
                        Some(self.bit_size_as_bytes[self.padding_left])
                    }
                }
            }
        }
    }

    struct BlockStream<'a, I> where I: IntoIterator<Item = u8> {
        paddedMessageStream: &'a mut ShaPadder<I>
    }

    impl<'a, I> BlockStream<'a, I> where I: IntoIterator<Item = u8> {
        pub fn new(paddedMessageStream: &mut ShaPadder<I>) -> BlockStream<I> {
            BlockStream{paddedMessageStream: paddedMessageStream}
        }
    }

    impl<'a, I> Iterator for BlockStream<'a, I> where I: IntoIterator<Item = u8> {
        type Item = [u32;16];

        fn next(&mut self) -> Option<Self::Item> {
            let blocksize = 64;
            let bytes: Vec<u8> = self.paddedMessageStream.take(blocksize).collect();
            if bytes.len() == 0 {
                None
            } else {
                if bytes.len() != blocksize {
                    panic!("Expected a block of {} bytes, but got only {}", blocksize, bytes.len());
                }

                let mut result: Self::Item = [0; 16];
                for u_no in 0..16 {
                    result[u_no] = u32_from_vec_slice(&bytes, u_no*4);
                }

                Some(result)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::ops::Range;

        extern crate rstest;
        use rstest::rstest;

        extern crate hex;

        #[rstest(input, expected_len, expected,
          case::empty( 0u8..0u8, 64, "80\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000"),

          case::one(   1u8..2u8, 64, "0180\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000008"),

          case::a_few( 1u8..16u8, 64, "0102030405060708090a0b0c0d0e0f80\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000078"),

          case::one_block( 0u8..0x37u8, 64, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f3031323334353680\
            \
            00000000000001b8"),

          case::one_block_and_one( 0u8..0x38u8, 128, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30313233343536378000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000001c0"),

          case::one_less_than_full_first_block( 0u8..0x3fu8, 128, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e80\
            0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000001f8"),


          case::first_block_just_full( 0u8..0x40u8, 128, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
            8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000200"),

          case::first_block_and_one( 0u8..0x41u8, 128, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
            4080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000208"),

          case::just_two_blocks( 0u8..0x77u8, 128, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
            404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f7071727374757680\
            00000000000003b8"),

          case::just_two_blocks_and_one( 0u8..0x78u8, 192, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
            404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f70717273747576778000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000003c0"),

          case::one_less_than_two_full_blocks( 0u8..0x7fu8, 192, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
            404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e80\
            0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000003f8"),

          case::two_full_blocks( 0u8..0x80u8, 192, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
            404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f\
            8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000400"),

          case::two_full_blocks_and_one( 0u8..0x81u8, 192, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
            404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f\
            8080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000408"),

        )]
        fn test_sha_padding(input: Range<u8>, expected: &str, expected_len: usize) {
            let result: Vec<u8> = ShaPadder::new(BlockParams{size: 64, reserve: 8}, input.into_iter()).collect();

            let hexstr = hex::encode(result);
            assert_eq!(hexstr.len(), 2*expected_len);
            assert_eq!(hexstr, expected);
        }

        #[rstest(input, expected,
          case::empty( 0u8..0u8,
            vec![
              [
                0x8000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000
             ]
            ]),

          case::one(   1u8..2u8,
              vec![
                [
                  0x0180_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0008
               ]
              ]),

          case::a_few( 1u8..16u8,
              vec![
                [
                  0x0102_0304, 0x0506_0708, 0x090a_0b0c, 0x0d0e_0f80,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0078
               ]
              ]),

          case::one_block( 0u8..0x37u8,
              vec![
                [
                  0x0001_0203, 0x0405_0607, 0x0809_0a0b, 0x0c0d_0e0f,
                  0x1011_1213, 0x1415_1617, 0x1819_1a1b, 0x1c1d_1e1f,
                  0x2021_2223, 0x2425_2627, 0x2829_2a2b, 0x2c2d_2e2f,
                  0x3031_3233, 0x3435_3680, 0x0000_0000, 0x0000_01b8
               ]
              ]),

          case::just_two_blocks( 0u8..0x77u8,
              vec![
                [
                  0x0001_0203, 0x0405_0607, 0x0809_0a0b, 0x0c0d_0e0f,
                  0x1011_1213, 0x1415_1617, 0x1819_1a1b, 0x1c1d_1e1f,
                  0x2021_2223, 0x2425_2627, 0x2829_2a2b, 0x2c2d_2e2f,
                  0x3031_3233, 0x3435_3637, 0x3839_3a3b, 0x3c3d_3e3f
                ], [
                  0x4041_4243, 0x4445_4647, 0x4849_4a4b, 0x4c4d_4e4f,
                  0x5051_5253, 0x5455_5657, 0x5859_5a5b, 0x5c5d_5e5f,
                  0x6061_6263, 0x6465_6667, 0x6869_6a6b, 0x6c6d_6e6f,
                  0x7071_7273, 0x7475_7680, 0x0000_0000, 0x0000_03b8
                ]
              ]),

          case::two_full_blocks( 0u8..0x80u8,
              vec![
                [
                  0x0001_0203, 0x0405_0607, 0x0809_0a0b, 0x0c0d_0e0f,
                  0x1011_1213, 0x1415_1617, 0x1819_1a1b, 0x1c1d_1e1f,
                  0x2021_2223, 0x2425_2627, 0x2829_2a2b, 0x2c2d_2e2f,
                  0x3031_3233, 0x3435_3637, 0x3839_3a3b, 0x3c3d_3e3f
                ], [
                  0x4041_4243, 0x4445_4647, 0x4849_4a4b, 0x4c4d_4e4f,
                  0x5051_5253, 0x5455_5657, 0x5859_5a5b, 0x5c5d_5e5f,
                  0x6061_6263, 0x6465_6667, 0x6869_6a6b, 0x6c6d_6e6f,
                  0x7071_7273, 0x7475_7677, 0x7879_7a7b, 0x7c7d_7e7f
                ],
                [
                  0x8000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
                  0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0400
               ]
              ]),

        )]
        fn test_sha_blocks(input: Range<u8>, expected: Vec<[u32;16]>) {
            let mut sha_padder = ShaPadder::new(BlockParams{size: 64, reserve: 8}, input.into_iter());
            let sha_blocks = BlockStream::new(&mut sha_padder);

            let result: Vec<[u32;16]> = sha_blocks.collect();

            assert_eq!(result, expected);
        }

    }

}

mod conv {

    pub fn u32_from_vec_slice(bytes: &Vec<u8>, offset: usize) -> u32 {
        let mut fourbytes: [u8;4] = [0; 4];
        fourbytes.copy_from_slice(&bytes[offset..offset+4]);

        u32::from_be_bytes(fourbytes)
    }

  #[cfg(test)]
  mod tests {
      use super::*;

      extern crate hex;

      #[test]
      fn test_u32_from_vec_slice() {
          let vec = hex::decode("0102030405060708").unwrap();

          assert_eq!(u32_from_vec_slice(&vec, 0), 0x01020304u32);
          assert_eq!(u32_from_vec_slice(&vec, 4), 0x05060708u32);

      }
  }
}
