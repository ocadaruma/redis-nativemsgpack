use std::ops::{Index, IndexMut};
use std::mem::size_of;

/// Abstraction layer for byte vector
pub trait ByteVector: Index<usize, Output = u8> + IndexMut<usize> {
    fn len(&self) -> usize;
}

pub struct MsgpackArray<T>
    where T : ByteVector {
    underlying: T,
    pub len: usize,
}

impl<T> MsgpackArray<T>
    where T : ByteVector {

    pub const FIRST_BYTE: u8 = 0xdd;

    pub fn get(&self, index: usize) -> i64 {
        // (9 byte (msgpack i64) * index) + (item count) + (array32 FIRST BYTE) + (u64 FIRST BYTE)
        let offset = (size_of::<i64>() + 1) * index + size_of::<u32>() + 1 + 1;

        (0..8).fold(0i64, |acc, i| acc | ((self.underlying[offset + i] as i64) << (i as i64 * 8)))
    }

    pub fn set(&mut self, index: usize, value: i64) {
        let offset = (size_of::<i64>() + 1) * index + size_of::<u32>() + 1 + 1;

        self.underlying[offset - 1] = 0xd3;

        for i in 0..8 {
            self.underlying[offset + i] = ((value >> (i as i64 * 8)) & 0xff) as u8;
        }
    }

    pub fn initialize(mut underlying: T) -> T {
        if underlying.len() < 1 + size_of::<u32>() {
            panic!("byte array is too short")
        }
        if (underlying.len() - 5) % 9 != 0 {
            panic!("illegal length")
        }

        underlying[0] = Self::FIRST_BYTE;
        for i in 0..size_of::<u32>() {
            underlying[1 + i] = 0;
        }

        let n = (underlying.len() - 5) / 9;
        for i in 0..4 {
            underlying[1 + i] = ((n >> (i * 4)) & 0xff) as u8;
        }

        underlying
    }

    pub fn parse(underlying: T) -> Option<MsgpackArray<T>> {
        if underlying.len() < 1 + size_of::<u32>() || (underlying.len() - 5) % 9 != 0 {
            return None
        }
        if underlying[0] != Self::FIRST_BYTE {
            return None
        }
        let n = (0..4).fold(0usize, |acc, i| acc | ((underlying[1 + i] as usize) << (i as usize * 4)));
        if (underlying.len() - 5) / 9 != n {
            return None
        }

        Some(MsgpackArray {
            underlying,
            len: n,
        })
    }

    pub fn binarysearch(&self, element: i64) -> SearchResult {
        if self.len < 1 {
            SearchResult::NotFound(0)
        } else {
            self._binarysearch(element, 0, self.len - 1)
        }
    }

    fn _binarysearch(&self, element: i64, start: usize, end: usize) -> SearchResult {
        if end - start < 1 {
            if self.get(start) == element {
                SearchResult::Found(start)
            } else {
                SearchResult::NotFound(if self.get(start) < element { start + 1 } else { start })
            }
        } else if end - start < 2 {
            if self.get(start) == element {
                SearchResult::Found(start)
            } else if self.get(end) == element {
                SearchResult::Found(end)
            } else {
                SearchResult::NotFound(if self.get(start) < element { end } else { start })
            }
        } else {
            let pivot = (start + end) / 2;
            if element < self.get(pivot) {
                self._binarysearch(element, start, pivot)
            } else {
                self._binarysearch(element, pivot, end)
            }
        }
    }
}

/// Vec based ByteVector impl. For unit testing purpose only.
impl ByteVector for Vec<u8> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum SearchResult {
    Found(usize),
    NotFound(usize),
}

#[cfg(test)]
mod tests {
    use super::SearchResult::*;
    use super::MsgpackArray;

    #[test]
    fn test_initialize() {
        assert_eq!(MsgpackArray::initialize(vec![0u8; 5])[0], 0xdd);
    }

    #[test]
    fn test_parse() {
        let result = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5]));

        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap().len, 0);

        let result = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 23]));

        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap().len, 2);

        assert_eq!(MsgpackArray::parse(vec![0u8; 5]).is_none(), true);
    }

    #[test]
    fn test_index() {
        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 14])).unwrap();

        assert_eq!(arr.get(0), 0);
        arr.set(0, 123456789);
        assert_eq!(arr.get(0), 123456789);

        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 905])).unwrap();

        assert_eq!(arr.get(99), 0);
        arr.set(99, 123456789);
        assert_eq!(arr.get(99), 123456789);
    }

    #[test]
    fn test_binarysearch() {
        // found
        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 6])).unwrap();
        arr.set(0, 2);
        arr.set(1, 3);
        arr.set(2, 5);
        arr.set(3, 7);
        arr.set(4, 11);
        arr.set(5, 13);
        assert_eq!(arr.binarysearch(7), Found(3));

        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 1])).unwrap();
        arr.set(0, 7);
        assert_eq!(arr.binarysearch(7), Found(0));

        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 3])).unwrap();
        arr.set(0, 2);
        arr.set(1, 3);
        arr.set(2, 5);
        assert_eq!(arr.binarysearch(2), Found(0));
        assert_eq!(arr.binarysearch(5), Found(2));

        // not found
        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 6])).unwrap();
        arr.set(0, 2);
        arr.set(1, 3);
        arr.set(2, 5);
        arr.set(3, 7);
        arr.set(4, 11);
        arr.set(5, 13);
        assert_eq!(arr.binarysearch(4), NotFound(2));

        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 1])).unwrap();
        arr.set(0, 7);
        assert_eq!(arr.binarysearch(3), NotFound(0));
        assert_eq!(arr.binarysearch(8), NotFound(1));

        let arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5])).unwrap();
        assert_eq!(arr.binarysearch(8), NotFound(0));
    }
}
