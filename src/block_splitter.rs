pub struct BlockStream<I> where I: IntoIterator<Item = u8> {
    message_iter: I::IntoIter,
    blocksize: usize
}

impl<'a, I> BlockStream<I> where I: IntoIterator<Item = u8> {
    pub fn new(blocksize: usize, message_iter: I) -> BlockStream<I> {
        BlockStream{blocksize, message_iter: message_iter.into_iter()}
    }
}

impl<'a, I> Iterator for BlockStream<I> where I: IntoIterator<Item = u8> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut remain = self.blocksize;
        while remain > 0 {
            match self.message_iter.next() {
                Some(b) => {
                    bytes.push(b);
                    remain -= 1;
                }
                None => if bytes.len()==0 {
                    return None
                } else {
                    return Some(bytes)
                }
            }
        }
        Some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    extern crate rstest;
    use rstest::rstest;

    extern crate hex;

    #[rstest(blocksize, input, expected,
        case::empty(8, 0u8..0u8, vec![]),

        case::one(8, 1u8..2u8,
            vec![
                vec![1u8]
            ]),

        case::a_few(8, 1u8..6u8,
            vec![
                vec![1u8, 2u8, 3u8, 4u8, 5u8]
            ]),

        case::one_block( 8, 0u8..0x8u8,
            vec![
                vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8]
            ]),

        case::one_block_and_one( 8, 0u8..0x9u8,
            vec![
                vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8],
            vec![8u8],
        ]),

        case::another_one_block_and_one( 4, 0u8..0x5u8,
            vec![
                vec![0u8, 1u8, 2u8, 3u8],
                vec![4u8],
            ]),

        case::two_blocks( 4, 0u8..0x8u8,
            vec![
                vec![0u8, 1u8, 2u8, 3u8],
                vec![4u8, 5u8, 6u8, 7u8],
            ]),

        case::two_blocks_and_one( 4, 0u8..9u8,
            vec![
                vec![0u8, 1u8, 2u8, 3u8],
                vec![4u8, 5u8, 6u8, 7u8],
                vec![8u8],
            ]),
        )]

    fn test_block_stream(blocksize: usize, input: Range<u8>, expected: Vec<Vec<u8>>) {
        let result = BlockStream::new(blocksize, input.into_iter()).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }

}
