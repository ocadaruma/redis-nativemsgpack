use std::mem::size_of;
use super::ByteVector;

/// Represents msgpack primitive
pub trait Primitive where Self : Sized + Ord {
    const FIRST_BYTE: u8;
    const SIZE: usize;

    fn read<T : ByteVector>(bytes: &T, from: usize) -> Option<Self>;

    fn write<T : ByteVector>(bytes: &mut T, from: usize, value: Self);
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Int64(pub i64);

impl Primitive for Int64 {
    const FIRST_BYTE: u8 = 0xd3;
    const SIZE: usize = size_of::<i64>();

    fn read<T: ByteVector>(bytes: &T, from: usize) -> Option<Self> {
        if bytes[from] != Self::FIRST_BYTE {
            None
        } else {
            let n = (0..Self::SIZE).fold(
                0i64,
                |a, i| a | (bytes[from + i + 1] as i64) << (i * Self::SIZE) as i64);
            Some(Int64(n))
        }
    }

    fn write<T: ByteVector>(bytes: &mut T, from: usize, value: Self) {
        bytes[from] = Self::FIRST_BYTE;

        let Self(n) = value;
        for i in 0..Self::SIZE {
            bytes[from + i + 1] = ((n >> (i as i64 * 8)) & 0xff) as u8;
        }
    }
}
