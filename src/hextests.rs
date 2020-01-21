#[cfg(test)]

//using the hex crate, so tests are only to check that its interface behaves roughly as expected

mod tests {
    extern crate rstest;

    extern crate hex;

    use rstest::rstest;

    #[rstest(input, expected,
      case::empty("", vec![]),
      case::one_byte("00", vec![0]),
      case::two_bytes("0000", vec![0, 0]),
      case::three_bytes("000000", vec![0,0,0]),
      case("56", vec![0x56]),
      case("1234", vec![0x12, 0x34]),
      case("12345678", vec![0x12, 0x34, 0x56, 0x78]),
      case("ff", vec![0xff]),
      case("ffeeddcc", vec![0xff, 0xee, 0xdd, 0xcc]),
      case("FF", vec![0xff])
    )]
    fn decode(input: &str, expected: Vec<u8>) {
        assert_eq!(hex::decode(input), Ok(expected));
    }

    #[rstest(input, expected,
        case::empty(vec![], ""),
        case(vec![0], "00"),
        case(vec![1], "01"),
        case(vec![0xf], "0f"),
        case(vec![0x10], "10"),
        case(vec![0xff], "ff"),
        case(vec![0, 0], "0000"),
        case(vec![1,2,3,4], "01020304"),
        case(vec![0xff, 0xff, 0xff], "ffffff")
    )]
    fn encode(input: Vec<u8>, expected: &str) {
        assert_eq!(hex::encode(input), expected);
    }
}
