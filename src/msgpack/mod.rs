pub mod format;

use std::ops::{Index, IndexMut};
use std::mem::size_of;
use format::*;

/// Abstraction layer for byte vector
pub trait ByteVector: Index<usize, Output = u8> + IndexMut<usize> {
    fn len(&self) -> usize;

    fn memmove(&mut self, dest: usize, src: usize, len: usize);

    fn realloc(&self, len: usize) -> Self;

    fn alloc(len: usize) -> Self;
}

pub struct MsgpackArray<T>
    where T : ByteVector {
    underlying: T,
}

impl<T> MsgpackArray<T>
    where T : ByteVector {

    pub fn len(&self) -> usize {
        match self.underlying[0] {
            b@(0x90..=0x9f) => (b - 0x90) as usize,
            0xdc => (0..2usize).fold(0, |a, i| a | ((self.underlying[1 + i] as usize) << i)),
            0xdd => (0..4usize).fold(0, |a, i| a | ((self.underlying[1 + i] as usize) << i)),
            _ => 0
        }
    }

    pub fn get<U : Primitive>(&self, index: usize) -> Option<U> {
        let len = self.len();
        if len <= index {
            return None;
        }
        let offset = match self.len() {
            0..=15 => (U::SIZE + 1) * index + 1,
            16..=65535 => (U::SIZE + 1) * index + 2 + 1,
            _ => (U::SIZE + 1) * index + 4 + 1,
        };

        U::read(&self.underlying, offset)
    }

    pub fn set<U : Primitive>(&mut self, index: usize, value: U) {
        let len = self.len();
        if len <= index {
            return;
        }
        let offset = match self.len() {
            0..=15 => (U::SIZE + 1) * index + 1,
            16..=65535 => (U::SIZE + 1) * index + 2 + 1,
            _ => (U::SIZE + 1) * index + 4 + 1,
        };

        U::write(&mut self.underlying, offset, value);
    }

    pub fn new() -> Self {
        let mut v = T::alloc(1);
        v[0] = 0x90;
        Self {
            underlying: v,
        }
    }

    pub fn parse<U : Primitive>(underlying: T) -> Option<Self> {
        if underlying.len() < 1 {
            return None;
        }
        match underlying[0] {
            b@(0x90..=0x9f) => {},
            0xdc => {
                if underlying.len() < 3 {
                    return None;
                }
                let len = (0..2usize).fold(0, |a, i| a | ((underlying[1 + i] as usize) << i));
                if underlying.len() != 3 + (U::SIZE + 1) * len {
                    return None;
                }
            },
            0xdd => {
                if underlying.len() < 5 {
                    return None;
                }
                let len = (0..4usize).fold(0, |a, i| a | ((underlying[1 + i] as usize) << i));
                if underlying.len() != 5 + (U::SIZE + 1) * len {
                    return None;
                }
            },
            _ => return None,
        };

        Some(Self {
            underlying,
        })
    }

    pub fn binarysearch<U : Primitive>(&self, element: U) -> SearchResult {
        if self.len() < 1 {
            SearchResult::NotFound(0)
        } else {
            self._binarysearch(element, 0, self.len() - 1)
        }
    }

    fn _binarysearch<U : Primitive>(&self, element: U, start: usize, end: usize) -> SearchResult {
        if end - start < 1 {
            if self.get::<U>(start).map_or_else(|| false, |e| e == element) {
                SearchResult::Found(start)
            } else {
                SearchResult::NotFound(if self.get::<U>(start).map_or_else(|| false,|e| e < element) {
                    start + 1
                } else {
                    start
                })
            }
        } else if end - start < 2 {
            if (self.get::<U>(start)).map_or_else(|| false, |e| e == element) {
                SearchResult::Found(start)
            } else if self.get::<U>(end).map_or_else(|| false, |e| e == element) {
                SearchResult::Found(end)
            } else {
                SearchResult::NotFound(if self.get::<U>(start).map_or_else(|| false, |e| e < element) {
                    end
                } else {
                    start
                })
            }
        } else {
            let pivot = (start + end) / 2;
            if self.get::<U>(pivot).map_or_else(|| false, |e| e > element) {
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

    fn memmove(&mut self, dest: usize, src: usize, len: usize) {
        for i in 0..len {
            self[dest + i] = self[src + i];
        }
    }

    fn realloc(&self, len: usize) -> Self {
        let mut ret = vec![];
        for i in 0..len {
            if i < self.len() {
                ret.push(self[i]);
            } else {
                ret.push(u8::default());
            }
        }
        ret
    }

    fn alloc(len: usize) -> Self {
        vec![0u8; len]
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
    use super::format::*;

    #[test]
    fn test_initialize() {
        let arr: MsgpackArray<Vec<u8>> = MsgpackArray::new();
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.underlying[0], 0x90);
    }

    #[test]
    fn test_parse() {
        let result: Option<MsgpackArray<Vec<u8>>> = MsgpackArray::parse::<Int64>(MsgpackArray::new().underlying);

        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap().len(), 0);

        let v = vec![0x92u8, 0xd3, 0, 0, 0, 0, 0, 0, 0, 0, 0xd3, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = MsgpackArray::parse::<Int64>(v);

        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap().len(), 2);

        assert_eq!(MsgpackArray::parse::<Int64>(vec![0u8; 5]).is_none(), true);
    }
//
//    #[test]
//    fn test_index() {
//        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 14])).unwrap();
//
//        assert_eq!(arr.get(0), 0);
//        arr.set(0, 123456789);
//        assert_eq!(arr.get(0), 123456789);
//
//        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 905])).unwrap();
//
//        assert_eq!(arr.get(99), 0);
//        arr.set(99, 123456789);
//        assert_eq!(arr.get(99), 123456789);
//    }
//
//    #[test]
//    fn test_binarysearch() {
//        // found
//        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 6])).unwrap();
//        arr.set(0, 2);
//        arr.set(1, 3);
//        arr.set(2, 5);
//        arr.set(3, 7);
//        arr.set(4, 11);
//        arr.set(5, 13);
//        assert_eq!(arr.binarysearch(7), Found(3));
//
//        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 1])).unwrap();
//        arr.set(0, 7);
//        assert_eq!(arr.binarysearch(7), Found(0));
//
//        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 3])).unwrap();
//        arr.set(0, 2);
//        arr.set(1, 3);
//        arr.set(2, 5);
//        assert_eq!(arr.binarysearch(2), Found(0));
//        assert_eq!(arr.binarysearch(5), Found(2));
//
//        // not found
//        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 6])).unwrap();
//        arr.set(0, 2);
//        arr.set(1, 3);
//        arr.set(2, 5);
//        arr.set(3, 7);
//        arr.set(4, 11);
//        arr.set(5, 13);
//        assert_eq!(arr.binarysearch(4), NotFound(2));
//
//        let mut arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5 + 9 * 1])).unwrap();
//        arr.set(0, 7);
//        assert_eq!(arr.binarysearch(3), NotFound(0));
//        assert_eq!(arr.binarysearch(8), NotFound(1));
//
//        let arr = MsgpackArray::parse(MsgpackArray::initialize(vec![0u8; 5])).unwrap();
//        assert_eq!(arr.binarysearch(8), NotFound(0));
//    }
}
