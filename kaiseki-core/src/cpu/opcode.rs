use num_traits::{FromBytes, FromPrimitive, ToBytes, ToPrimitive, Unsigned};
use std::{fmt, ops};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpcodeError {
    #[error("{0} index {1} out of bounds, must be [0, {2})")]
    IndexOutOfBounds(&'static str, usize, usize),
}

pub type Result<T> = std::result::Result<T, OpcodeError>;

pub trait OpcodeValue:
    Copy
    + fmt::Debug
    + fmt::Display
    + Unsigned
    + ToBytes
    + ToPrimitive
    + FromBytes
    + FromPrimitive
    + ops::BitAnd<Self, Output = Self>
    + ops::Shl<usize, Output = Self>
    + ops::Shr<usize, Output = Self>
{
    const MASK_BIT: Self;
    const MASK_BYTE: Self;
    const MASK_NYBBLE: Self;

    const WIDTH_BITS: usize = Self::WIDTH_BYTES * 8;
    const WIDTH_BYTES: usize = std::mem::size_of::<Self>();
    const WIDTH_NYBBLES: usize = Self::WIDTH_BYTES * 2;
}

impl OpcodeValue for u8 {
    const MASK_BIT: Self = 0x01;
    const MASK_BYTE: Self = 0xFF;
    const MASK_NYBBLE: Self = 0x0F;
}
impl OpcodeValue for u16 {
    const MASK_BIT: Self = 0x01;
    const MASK_BYTE: Self = 0xFF;
    const MASK_NYBBLE: Self = 0x0F;
}
impl OpcodeValue for u32 {
    const MASK_BIT: Self = 0x01;
    const MASK_BYTE: Self = 0xFF;
    const MASK_NYBBLE: Self = 0x0F;
}
impl OpcodeValue for u64 {
    const MASK_BIT: Self = 0x01;
    const MASK_BYTE: Self = 0xFF;
    const MASK_NYBBLE: Self = 0x0F;
}

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opcode<T: OpcodeValue> {
    value: T,
}

impl<T: OpcodeValue> Opcode<T> {
    fn extract(
        &self,
        idx_type: &'static str,
        idx: usize,
        width: usize,
        mask: T,
        shift: usize,
    ) -> Result<u8> {
        let max_idx = width - 1;
        if idx > max_idx {
            return Err(OpcodeError::IndexOutOfBounds(idx_type, idx, width));
        }
        let mask = mask << shift;
        let result = (self.value & mask) >> shift;
        Ok(result
            .to_u8()
            .expect("extracted value can be converted to u8"))
    }

    pub fn value(&self) -> T {
        self.value
    }

    pub fn get_bit(&self, idx: usize) -> u8 {
        self.try_get_bit(idx).unwrap()
    }

    pub fn get_byte(&self, idx: usize) -> u8 {
        self.try_get_byte(idx).unwrap()
    }

    pub fn get_nybble(&self, idx: usize) -> u8 {
        self.try_get_nybble(idx).unwrap()
    }

    pub fn try_get_bit(&self, idx: usize) -> Result<u8> {
        self.extract("bit", idx, T::WIDTH_BITS, T::MASK_BIT, idx)
    }

    pub fn try_get_byte(&self, idx: usize) -> Result<u8> {
        self.extract("byte", idx, T::WIDTH_BYTES, T::MASK_BYTE, idx * 8)
    }

    pub fn try_get_nybble(&self, idx: usize) -> Result<u8> {
        self.extract("nybble", idx, T::WIDTH_NYBBLES, T::MASK_NYBBLE, idx * 4)
    }
}

fn slice_it<const N: usize>(bytes: &[u8]) -> [u8; N] {
    bytes[0..N].try_into().expect("convert &[u8] to [u8; N]")
}

pub type Opcode8 = Opcode<u8>;
impl fmt::Debug for Opcode<u8> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:02X}", self.value))
    }
}
impl Opcode<u8> {
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u8::from_be_bytes(slice_it(bytes)),
        }
    }

    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u8::from_le_bytes(slice_it(bytes)),
        }
    }
}

pub type Opcode16 = Opcode<u16>;
impl Opcode<u16> {
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u16::from_be_bytes(slice_it(bytes)),
        }
    }

    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u16::from_le_bytes(slice_it(bytes)),
        }
    }
}
impl fmt::Debug for Opcode<u16> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:04X}", self.value))
    }
}

pub type Opcode32 = Opcode<u32>;
impl fmt::Debug for Opcode<u32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:08X}", self.value))
    }
}
impl Opcode<u32> {
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u32::from_be_bytes(slice_it(bytes)),
        }
    }

    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u32::from_le_bytes(slice_it(bytes)),
        }
    }
}

pub type Opcode64 = Opcode<u64>;
impl fmt::Debug for Opcode<u64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:016X}", self.value))
    }
}
impl Opcode<u64> {
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u64::from_be_bytes(slice_it(bytes)),
        }
    }

    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        Self {
            value: u64::from_le_bytes(slice_it(bytes)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALUE8: u8 = (VALUE64 & u8::MAX as u64) as u8;
    const VALUE16: u16 = (VALUE64 & u16::MAX as u64) as u16;
    const VALUE32: u32 = (VALUE64 & u32::MAX as u64) as u32;
    const VALUE64: u64 = 0x0123456789ABCDEF;

    const fn create_opcodes() -> (Opcode8, Opcode16, Opcode32, Opcode64) {
        let op8 = Opcode8 { value: VALUE8 };
        let op16 = Opcode16 { value: VALUE16 };
        let op32 = Opcode32 { value: VALUE32 };
        let op64 = Opcode64 { value: VALUE64 };
        (op8, op16, op32, op64)
    }

    fn assert_expected<T: OpcodeValue>(
        idx_type: &'static str,
        idx: usize,
        width: usize,
        mask: T,
        shift: usize,
        result: Result<u8>,
    ) {
        if idx < width {
            let mask = mask.to_u64().unwrap();
            let expected: u8 = ((VALUE64 & (mask << shift)) >> shift).to_u8().unwrap();
            assert_eq!(result, Ok(expected));
        } else {
            assert_eq!(
                result,
                Err(OpcodeError::IndexOutOfBounds(idx_type, idx, width))
            );
        }
    }

    fn assert_bits_expected<T: OpcodeValue>(idx: usize, result: Result<u8>) {
        assert_expected::<T>("bit", idx, T::WIDTH_BITS, T::MASK_BIT, idx, result)
    }

    #[test]
    fn test_opcode_bits() {
        let (op8, op16, op32, op64) = create_opcodes();
        for idx in 0..=u64::WIDTH_BITS {
            let res = op8.try_get_bit(idx);
            assert_bits_expected::<u8>(idx, res);

            let res = op16.try_get_bit(idx);
            assert_bits_expected::<u16>(idx, res);

            let res = op32.try_get_bit(idx);
            assert_bits_expected::<u32>(idx, res);

            let res = op64.try_get_bit(idx);
            assert_bits_expected::<u64>(idx, res);
        }
    }

    fn assert_bytes_expected<T: OpcodeValue>(idx: usize, result: Result<u8>) {
        assert_expected::<T>("byte", idx, T::WIDTH_BYTES, T::MASK_BYTE, idx * 8, result)
    }

    #[test]
    fn test_opcode_bytes() {
        let (op8, op16, op32, op64) = create_opcodes();
        for idx in 0..=u64::WIDTH_BYTES {
            let res = op8.try_get_byte(idx);
            assert_bytes_expected::<u8>(idx, res);

            let res = op16.try_get_byte(idx);
            assert_bytes_expected::<u16>(idx, res);

            let res = op32.try_get_byte(idx);
            assert_bytes_expected::<u32>(idx, res);

            let res = op64.try_get_byte(idx);
            assert_bytes_expected::<u64>(idx, res);
        }
    }

    fn assert_nybbles_expected<T: OpcodeValue>(idx: usize, result: Result<u8>) {
        assert_expected::<T>(
            "nybble",
            idx,
            T::WIDTH_NYBBLES,
            T::MASK_NYBBLE,
            idx * 4,
            result,
        )
    }

    #[test]
    fn test_opcode_nybbles() {
        let (op8, op16, op32, op64) = create_opcodes();
        for idx in 0..=u64::WIDTH_NYBBLES {
            let res = op8.try_get_nybble(idx);
            assert_nybbles_expected::<u8>(idx, res);

            let res = op16.try_get_nybble(idx);
            assert_nybbles_expected::<u16>(idx, res);

            let res = op32.try_get_nybble(idx);
            assert_nybbles_expected::<u32>(idx, res);

            let res = op64.try_get_nybble(idx);
            assert_nybbles_expected::<u64>(idx, res);
        }
    }
}
