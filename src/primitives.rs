#![macro_use]

macro_rules! rotr {
    ($x:expr, $n: literal) =>  { ($x >> $n) | ($x << (std::mem::size_of_val(&$x)*8 - $n)) }
}

macro_rules! ch {
    ($x:expr, $y:expr, $z:expr) => {($x & $y) ^ (!$x & $z)}
}

macro_rules! maj {
    ($x:expr, $y:expr, $z:expr) => {($x & $y) ^ ($x & $z) ^ ($y & $z)}
}

macro_rules! Sigma {
    ($x:expr, $n1:literal, $n2:literal, $n3:literal) => {
        rotr!($x, $n1) ^ rotr!($x, $n2) ^ rotr!($x, $n3)
    }
}

macro_rules! sigma {
    ($x:expr, $n1:literal, $n2:literal, $n3:literal) => {
        rotr!($x, $n1) ^ rotr!($x, $n2) ^ ($x >> $n3)
    }
}

#[cfg(test)]
mod tests_32 {
    extern crate rstest;
    use rstest::rstest;

    #[rstest(x, y, z, res,
        case::mix(0x00ff00ff, 0xaaaaaaaa, 0x55555555, 0x55aa55aa),
        case::remix(0xff00ff00, 0xaaaaaaaa, 0x55555555, 0xaa55aa55),
        case::pick(0xaaaaaaaa,0xaaaaaaaa,0x55555555,0xffffffff),
        case::choose(0x55555555,0xaaaaaaaa,0x55555555,0),
        case::swap(0xaaaaaaaa,0x55555555,0xaaaaaaaa,0)
    )]
    fn test_ch(x:u32, y:u32, z:u32, res:u32) {
        assert_eq!(ch!(x,y,z), res);
    }

    #[rstest(x, y, z, res,
        case::mix_pattern(0x0f0f0f0f,0x00ff00ff,0x0000ffff,0x00f0fff),
        case::tiebreaking(0x0000ffff,0xa5a5a5a5,0xffff0000,0xa5a5a5a5),
    )]
    fn test_maj(x:u32, y:u32, z:u32, res:u32) {
        assert_eq!(maj!(x,y,z), res);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Sigma_higend() {
        assert_eq!(Sigma!(0xffff0000u32, 1, 4, 8),  0x70ff8f00);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Sigma_lowend() {
        assert_eq!(Sigma!(0x0000ffffu32, 1, 4, 8), 0x8f0070ff);
    }

    #[test]
    fn test_sigma_highend() {
        assert_eq!(sigma!(0xffff0000u32, 1, 4, 8), 0x70ff8f00);
    }

    #[test]
    fn test_sigma_lowend() {
        assert_eq!(sigma!(0x0000ffffu32, 1, 4, 8), 0x700070ff);
    }
}

#[cfg(test)]
mod tests_64 {
    extern crate rstest;
    use rstest::rstest;

    #[rstest(x, y, z, res,
        case::mix(0x00ff00ff00ff00ff, 0xaaaaaaaaaaaaaaaa, 0x5555555555555555, 0x55aa55aa55aa55aa),
        case::remix(0xff00ff00ff00ff00, 0xaaaaaaaaaaaaaaaa, 0x5555555555555555, 0xaa55aa55aa55aa55),
        case::pick(0xaaaaaaaaaaaaaaaa,0xaaaaaaaaaaaaaaaa,0x5555555555555555,0xffffffffffffffff),
        case::choose(0x5555555555555555,0xaaaaaaaaaaaaaaaa,0x5555555555555555,0),
        case::swap(0xaaaaaaaaaaaaaaaa,0x5555555555555555,0xaaaaaaaaaaaaaaaa,0)
    )]
    fn test_ch(x:u64, y:u64, z:u64, res:u64) {
        assert_eq!(ch!(x,y,z), res);
    }

    #[rstest(x, y, z, res,
        case::mix_pattern(0x0f0f0f0f0f0f0f0f,0x00ff00ff00ff00ff,0x0000ffff0000ffff,0x000f0fff000f0fff),
        case::tiebreaking(0x0000ffff0000ffff,0xa5a5a5a5a5a5a5a5,0xffff0000ffff0000,0xa5a5a5a5a5a5a5a5),
    )]
    fn test_maj(x:u64, y:u64, z:u64, res:u64) {
        assert_eq!(maj!(x,y,z), res);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Sigma_higend() {
        assert_eq!(Sigma!(0xffff0000ffff0000u64, 1, 4, 8),  0x70ff8f0070ff8f00);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Sigma_lowend() {
        assert_eq!(Sigma!(0x0000ffff0000ffffu64, 1, 4, 8), 0x8f0070ff8f0070ff);
    }

    #[test]
    fn test_sigma_highend() {
        assert_eq!(sigma!(0xffff0000ffff0000u64, 1, 4, 8), 0x70ff8f0070ff8f00);
    }

    #[test]
    fn test_sigma_lowend() {
        assert_eq!(sigma!(0x0000ffff0000ffffu64, 1, 4, 8), 0x700070ff8f0070ff);
    }
}
