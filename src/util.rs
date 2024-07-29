///
/// This macro is extremely fast.
///
#[macro_export]
macro_rules! popcnt64 {
    ($e:expr) => {
        unsafe { core::arch::x86_64::_popcnt64($e as i64) }
    };
}

///
/// This macro is extremely fast.
///
#[macro_export]
macro_rules! lzcnt64 {
    ($e:expr) => {
        unsafe { core::arch::x86_64::_lzcnt_u64($e) }
    };
}

///
/// This macro is extremely fast.
///
#[macro_export]
macro_rules! tzcnt64 {
    ($e:expr) => {
        unsafe { core::arch::x86_64::_tzcnt_u64($e) }
    };
}

///
/// This macro is extremely fast.
///
#[macro_export]
macro_rules! blsmsk64 {
    ($e:expr) => {
        unsafe { core::arch::x86_64::_blsmsk_u64($e) }
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test_popcnt64() {
        assert_eq!(popcnt64!(0x01010101), 4);
    }

    #[test]
    fn test_lzcnt64() {
        assert_eq!(lzcnt64!(0x000FFFFFFFFFFFFF), 12);
    }

    #[test]
    fn test_tzcnt64() {
        assert_eq!(tzcnt64!(0xFFFFFFFFFFFFF000), 12);
    }

    #[test]
    fn test_blsmsk64() {
        assert_eq!(blsmsk64!(0xFFFFFFFFFFFFF000) & 0xFFFFFFFFFFFFF000, 1 << 12);
    }
}
