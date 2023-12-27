use std::{fmt, result::Result, convert::Infallible};
use num_traits::{Num, Unsigned};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Register<T> where T: Unsigned + Copy {
    value: T,
}

impl<T: Unsigned + Copy> Register<T> {
    fn read(&self) -> T {
        self.value
    }

    fn write(&mut self, value: T) {
        self.value = value;
    }

    fn try_read_as<V: Num + From<T>>(&self) -> Result<V, Infallible> {
        V::try_from(self.value)
    }
}

pub type Reg8 = Register<u8>;
impl fmt::Debug for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:02X}", self.value))
    }
}

pub type Reg16 = Register<u16>;
impl fmt::Debug for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:04X}", self.value))
    }
}

pub type Reg32 = Register<u32>;
impl fmt::Debug for Reg32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:08X}", self.value))
    }
}

pub type Reg64 = Register<u64>;
impl fmt::Debug for Reg64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:016X}", self.value))
    }
}

pub trait RegisterSet {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reg8() {
        let mut reg = Reg8 { value: 0 };
        assert_eq!(reg.read(), 0);
        reg.write(1);
        assert_eq!(reg.read(), 1);
        reg.write(u8::MAX);
        assert_eq!(reg.read(), u8::MAX);
        let word: u16 = reg.try_read_as().unwrap();
        assert_eq!(word, u8::MAX.into());
        let double_word: u32 = reg.try_read_as().unwrap();
        assert_eq!(double_word, u8::MAX.into());
        let quadword: u64 = reg.try_read_as().unwrap();
        assert_eq!(quadword, u8::MAX.into());
        let usize: usize = reg.try_read_as().unwrap();
        assert_eq!(usize, u8::MAX.into());

        let signed_word: i16 = reg.try_read_as().unwrap();
        assert_eq!(signed_word, u8::MAX.into());
        let signed_double_word: i32 = reg.try_read_as().unwrap();
        assert_eq!(signed_double_word, u8::MAX.into());
        let signed_quadword: i64 = reg.try_read_as().unwrap();
        assert_eq!(signed_quadword, u8::MAX.into());
        let isize: isize = reg.try_read_as().unwrap();
        assert_eq!(isize, u8::MAX.into());

        let float32: f32 = reg.try_read_as().unwrap();
        assert_eq!(float32, u8::MAX.into());
        let float64: f64 = reg.try_read_as().unwrap();
        assert_eq!(float64, u8::MAX.into());
    }

    #[test]
    fn test_reg16() {
        let mut reg = Reg16 { value: 0 };
        assert_eq!(reg.read(), 0);
        reg.write(1);
        assert_eq!(reg.read(), 1);
        reg.write(u16::MAX);
        assert_eq!(reg.read(), u16::MAX);
        let word: u16 = reg.try_read_as().unwrap();
        assert_eq!(word, u16::MAX.into());
        let double_word: u32 = reg.try_read_as().unwrap();
        assert_eq!(double_word, u16::MAX.into());
        let quadword: u64 = reg.try_read_as().unwrap();
        assert_eq!(quadword, u16::MAX.into());
        let usize: usize = reg.try_read_as().unwrap();
        assert_eq!(usize, u16::MAX.into());

        let signed_double_word: i32 = reg.try_read_as().unwrap();
        assert_eq!(signed_double_word, u16::MAX.into());
        let signed_quadword: i64 = reg.try_read_as().unwrap();
        assert_eq!(signed_quadword, u16::MAX.into());

        let float32: f32 = reg.try_read_as().unwrap();
        assert_eq!(float32, u16::MAX.into());
        let float64: f64 = reg.try_read_as().unwrap();
        assert_eq!(float64, u16::MAX.into());
    }

    #[test]
    fn test_reg32() {
        let mut reg = Reg32 { value: 0 };
        assert_eq!(reg.read(), 0);
        reg.write(1);
        assert_eq!(reg.read(), 1);
        reg.write(u32::MAX);
        assert_eq!(reg.read(), u32::MAX);
        let double_word: u32 = reg.try_read_as().unwrap();
        assert_eq!(double_word, u32::MAX.into());
        let quadword: u64 = reg.try_read_as().unwrap();
        assert_eq!(quadword, u32::MAX.into());

        let signed_quadword: i64 = reg.try_read_as().unwrap();
        assert_eq!(signed_quadword, u32::MAX.into());

        let float64: f64 = reg.try_read_as().unwrap();
        assert_eq!(float64, u32::MAX.into());
    }

    #[test]
    fn test_reg64() {
        let mut reg = Reg64 { value: 0 };
        assert_eq!(reg.read(), 0);
        reg.write(1);
        assert_eq!(reg.read(), 1);
        reg.write(u64::MAX);
        assert_eq!(reg.read(), u64::MAX);
        let quadword: u64 = reg.try_read_as().unwrap();
        assert_eq!(quadword, u64::MAX.into());
    }
}