mod iters {
    use std::mem;

    pub struct ShaPadder<I> where I: IntoIterator<Item = u8> {
        message_count: usize,
        message_done: bool,
        padding_left: usize,
        size_bytes: [u8;8], //size is measure in bits, but these are the bytes that make up the number
        message_iter: I::IntoIter,
    }

    impl<I> ShaPadder<I> where I: IntoIterator<Item = u8> {
        pub fn new(message: I) -> ShaPadder<I> {
            ShaPadder{message_count: 0, message_done: false, message_iter: message.into_iter(),
                size_bytes: [0u8;8], padding_left: 0}
        }
    }

    impl<I> Iterator for ShaPadder<I> where I: IntoIterator<Item = u8> {
        type Item = u8;

        fn next(&mut self) -> Option<u8> {
            if self.message_done {
                if self.padding_left == 0 {
                    None
                } else {
                    self.padding_left -= 1;
                    if self.padding_left < mem::size_of::<u64>() {
                        // println!("padding.left = {}, value = {}", self.padding_left, self.size_bytes[self.padding_left]);

                        Some(self.size_bytes[self.padding_left])
                    } else {
                        Some(0)
                    }
                }
            } else {
                match self.message_iter.next() {
                    Some(b) => {
                        self.message_count += 1;
                        Some(b)
                    }
                    None => {
                        self.message_done = true;
                        self.size_bytes = ((self.message_count*8) as u64).to_le_bytes(); //the size is measure in bits, but counted in bytes

                        let block_size = 64;
                        self.padding_left = block_size - (self.message_count % block_size) - 1;
                        if self.padding_left < mem::size_of::<u64>() {
                            self.padding_left += block_size;
                        }

                        // println!("count = {}", self.message_count);
                        // println!("as bytes = {:?}", self.size_bytes);

                        Some(0x80u8)
                    }
                }
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
            let result: Vec<u8> = ShaPadder::new(input.into_iter()).collect();

            let hexstr = hex::encode(result);
            assert_eq!(hexstr.len(), 2*expected_len);
            assert_eq!(hexstr, expected);
        }
    }

}

