pub mod format;

use format::*;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// Abstraction layer for resizable byte array
pub trait ByteVector: Index<usize, Output = u8> + IndexMut<usize>
where
    Self: Sized,
{
    type AllocErr;

    fn len(&self) -> usize;

    fn memmove(&mut self, dest: usize, src: usize, len: usize);

    fn realloc(&self, len: usize) -> Result<Self, Self::AllocErr>;
}

/// Represents msgpack array of single type elements
pub struct MsgpackArray<T, U>
where
    T: ByteVector,
    U: Primitive,
{
    underlying: T,
    element_type: PhantomData<U>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ArrayHeader {
    Fix(usize),
    Array16(usize),
    Array32(usize),
}

impl ArrayHeader {
    pub fn len(&self) -> usize {
        match self {
            &ArrayHeader::Fix(n) => n,
            &ArrayHeader::Array16(n) => n,
            &ArrayHeader::Array32(n) => n,
        }
    }

    pub fn header_bytes(&self) -> usize {
        match self {
            ArrayHeader::Fix(_) => 1,
            ArrayHeader::Array16(_) => 3,
            ArrayHeader::Array32(_) => 5,
        }
    }

    pub fn from_len(len: usize) -> Self {
        match len {
            0..=15 => ArrayHeader::Fix(len),
            16..=65535 => ArrayHeader::Array16(len),
            _ => ArrayHeader::Array32(len),
        }
    }

    pub fn byte_offset<U: Primitive>(&self, index: usize) -> usize {
        match self {
            ArrayHeader::Fix(_) => (U::SIZE + 1) * index + 1,
            ArrayHeader::Array16(_) => (U::SIZE + 1) * index + 2 + 1,
            ArrayHeader::Array32(_) => (U::SIZE + 1) * index + 4 + 1,
        }
    }

    pub fn total_bytes<U: Primitive>(&self) -> usize {
        match self {
            &ArrayHeader::Fix(n) => n * (U::SIZE + 1) + 1,
            &ArrayHeader::Array16(n) => n * (U::SIZE + 1) + 2 + 1,
            &ArrayHeader::Array32(n) => n * (U::SIZE + 1) + 4 + 1,
        }
    }
}

impl<T, U> MsgpackArray<T, U>
where
    T: ByteVector,
    U: Primitive,
{
    pub fn header(&self) -> ArrayHeader {
        match self.underlying[0] {
            b @ (0x90..=0x9f) => ArrayHeader::Fix((b - 0x90) as usize),
            0xdc => ArrayHeader::Array16((0..2usize).fold(0, |a, i| {
                a | ((self.underlying[1 + i] as usize) << ((1 - i) * 8))
            })),
            _ => ArrayHeader::Array32((0..4usize).fold(0, |a, i| {
                a | ((self.underlying[1 + i] as usize) << ((3 - i) * 8))
            })),
        }
    }

    pub fn delete_at(&mut self, index: usize) -> Result<(), T::AllocErr> {
        let current_header = self.header();

        if index >= current_header.len() {
            // early return if index out of range
            return Ok(());
        }
        if index < current_header.len() - 1 {
            self.underlying.memmove(
                current_header.header_bytes() + index * (U::SIZE + 1),
                current_header.header_bytes() + (index + 1) * (U::SIZE + 1),
                (current_header.len() - index - 1) * (U::SIZE + 1),
            );
        }

        let new_len = self.header().len() - 1;
        let new_header = ArrayHeader::from_len(new_len);
        let new_bytes = new_header.total_bytes::<U>();

        if current_header.header_bytes() != new_header.header_bytes() && new_header.len() > 0 {
            self.underlying.memmove(
                new_header.header_bytes(),
                current_header.header_bytes(),
                new_bytes,
            );
        }

        self.underlying = self.underlying.realloc(new_bytes)?;
        self.write_header(new_header);

        Ok(())
    }

    pub fn insert_at(&mut self, index: usize, element: U) -> Result<(), T::AllocErr> {
        let new_len = self.header().len() + 1;

        let current_header = self.header();
        let new_header = ArrayHeader::from_len(new_len);

        let new_bytes = new_header.total_bytes::<U>();
        self.underlying = self.underlying.realloc(new_bytes)?;

        if current_header.header_bytes() != new_header.header_bytes() && current_header.len() > 0 {
            self.underlying.memmove(
                new_header.header_bytes(),
                current_header.header_bytes(),
                current_header.total_bytes::<U>(),
            );
        }

        if index < current_header.len() {
            self.underlying.memmove(
                new_header.header_bytes() + (index + 1) * (U::SIZE + 1),
                new_header.header_bytes() + index * (U::SIZE + 1),
                (current_header.len() - index) * (U::SIZE + 1),
            );
        }

        self.write_header(new_header);
        self.set(index, element);

        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<U> {
        if self.header().len() <= index {
            return None;
        }

        U::read(&self.underlying, self.header().byte_offset::<U>(index))
    }

    pub fn set(&mut self, index: usize, value: U) {
        if self.header().len() <= index {
            return;
        }

        let byte_offset = self.header().byte_offset::<U>(index);
        U::write(&mut self.underlying, byte_offset, value);
    }

    pub fn new<F>(allocator: F) -> Result<Self, T::AllocErr>
    where
        F: FnOnce(usize) -> Result<T, T::AllocErr>,
    {
        let mut v = allocator(1)?;
        v[0] = 0x90;

        Ok(Self {
            underlying: v,
            element_type: PhantomData,
        })
    }

    pub fn parse(underlying: T) -> Option<Self> {
        if underlying.len() < 1 {
            return None;
        }
        let len = match underlying[0] {
            b @ (0x90..=0x9f) => (b - 0x90) as usize,
            0xdc => {
                if underlying.len() < 3 {
                    return None;
                }
                (0..2usize).fold(0, |a, i| {
                    a | ((underlying[1 + i] as usize) << ((1 - i) * 8))
                })
            }
            0xdd => {
                if underlying.len() < 5 {
                    return None;
                }
                (0..4usize).fold(0, |a, i| {
                    a | ((underlying[1 + i] as usize) << ((3 - i) * 8))
                })
            }
            _ => return None,
        };

        if underlying.len() == ArrayHeader::from_len(len).total_bytes::<U>() {
            Some(Self {
                underlying,
                element_type: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn binarysearch(&self, element: U) -> SearchResult {
        let len = self.header().len();
        if len < 1 {
            SearchResult::NotFound(0)
        } else {
            self._binarysearch(element, 0, len)
        }
    }

    fn write_header(&mut self, header: ArrayHeader) {
        match header {
            ArrayHeader::Fix(n) => self.underlying[0] = 0x90 + n as u8,
            ArrayHeader::Array16(n) => {
                self.underlying[0] = 0xdc;
                self.underlying[1] = ((n >> 8) & 0xff) as u8;
                self.underlying[2] = (n & 0xff) as u8;
            }
            ArrayHeader::Array32(n) => {
                self.underlying[0] = 0xdd;
                self.underlying[1] = ((n >> 24) & 0xff) as u8;
                self.underlying[2] = ((n >> 16) & 0xff) as u8;
                self.underlying[3] = ((n >> 8) & 0xff) as u8;
                self.underlying[4] = (n & 0xff) as u8;
            }
        }
    }

    fn _binarysearch(&self, element: U, start: usize, size: usize) -> SearchResult {
        if size > 1 {
            let half = size / 2;
            let mid = start + half;

            let e = self.get(mid).unwrap();
            let next_start = if element < e { start } else { mid };
            self._binarysearch(element, next_start, size - half)
        } else {
            let e = self.get(start).unwrap();
            if element == e {
                SearchResult::Found(start)
            } else {
                SearchResult::NotFound(start + if element > e { 1 } else { 0 })
            }
        }
    }
}

/// Vec based ByteVector impl. For unit testing purpose only.
impl ByteVector for Vec<u8> {
    type AllocErr = ();

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn memmove(&mut self, dest: usize, src: usize, len: usize) {
        if dest > src {
            for i in (0..len).rev() {
                self[dest + i] = self[src + i];
            }
        } else {
            for i in 0..len {
                self[dest + i] = self[src + i];
            }
        }
    }

    fn realloc(&self, len: usize) -> Result<Self, Self::AllocErr> {
        let mut ret = vec![];
        for i in 0..len {
            if i < self.len() {
                ret.push(self[i]);
            } else {
                ret.push(u8::default());
            }
        }
        Ok(ret)
    }
}

#[derive(Debug, PartialEq)]
pub enum SearchResult {
    Found(usize),
    NotFound(usize),
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::format::*;
    use super::ArrayHeader;
    use super::MsgpackArray;
    use super::SearchResult::*;

    #[test]
    fn test_initialize() {
        let arr: MsgpackArray<Vec<u8>, Int64> =
            MsgpackArray::new(|len| Ok(vec![0u8; len])).unwrap();
        assert_eq!(arr.header(), ArrayHeader::Fix(0));
        assert_eq!(arr.underlying[0], 0x90);
    }

    #[test]
    fn test_parse() {
        let result: Option<MsgpackArray<Vec<u8>, Int64>> = MsgpackArray::parse(
            MsgpackArray::<Vec<u8>, Int64>::new(|len| Ok(vec![0u8; len]))
                .unwrap()
                .underlying,
        );

        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap().header(), ArrayHeader::Fix(0));

        let v = vec![
            0x92u8, 0xd3, 0, 0, 0, 0, 0, 0, 0, 0, 0xd3, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let result: Option<MsgpackArray<Vec<u8>, Int64>> = MsgpackArray::parse(v);

        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap().header(), ArrayHeader::Fix(2));

        assert_eq!(
            MsgpackArray::<Vec<u8>, Int64>::parse(vec![0u8; 5]).is_none(),
            true
        );
    }

    #[test]
    fn test_index() {
        let mut v = vec![0x91u8];
        for _ in 0..1 {
            v.push(0xd3);
            for _ in 0..8 {
                v.push(0);
            }
        }
        let mut arr = MsgpackArray::parse(v).unwrap();

        assert_eq!(arr.get(0), Some(Int64(0)));
        arr.set(0, Int64(123456789));
        assert_eq!(arr.get(0), Some(Int64(123456789)));

        let mut v = vec![0xdc, 0, 100u8];
        for _ in 0..100 {
            v.push(0xd3);
            for _ in 0..8 {
                v.push(0);
            }
        }
        let mut arr = MsgpackArray::parse(v).unwrap();

        assert_eq!(arr.get(99), Some(Int64(0)));
        arr.set(99, Int64(123456789));
        assert_eq!(arr.get(99), Some(Int64(123456789)));
    }

    #[test]
    fn test_binarysearch_found() {
        // found
        let mut v = vec![0x96];
        for _ in 0..6 {
            v.push(0xd3);
            for _ in 0..8 {
                v.push(0);
            }
        }
        let mut arr = MsgpackArray::parse(v).unwrap();
        arr.set(0, Int64(2));
        arr.set(1, Int64(3));
        arr.set(2, Int64(5));
        arr.set(3, Int64(7));
        arr.set(4, Int64(11));
        arr.set(5, Int64(13));
        assert_eq!(arr.binarysearch(Int64(7)), Found(3));

        let mut v = vec![0x91];
        for _ in 0..1 {
            v.push(0xd3);
            for _ in 0..8 {
                v.push(0);
            }
        }
        let mut arr = MsgpackArray::parse(v).unwrap();
        arr.set(0, Int64(7));
        assert_eq!(arr.binarysearch(Int64(7)), Found(0));

        let mut v = vec![0x93];
        for _ in 0..3 {
            v.push(0xd3);
            for _ in 0..8 {
                v.push(0);
            }
        }
        let mut arr = MsgpackArray::parse(v).unwrap();
        arr.set(0, Int64(2));
        arr.set(1, Int64(3));
        arr.set(2, Int64(5));
        assert_eq!(arr.binarysearch(Int64(2)), Found(0));
        assert_eq!(arr.binarysearch(Int64(5)), Found(2));
    }

    #[test]
    fn test_binarysearch_notfound() {
        // not found
        let mut v = vec![0x96];
        for _ in 0..6 {
            v.push(0xd3);
            for _ in 0..8 {
                v.push(0);
            }
        }
        let mut arr = MsgpackArray::parse(v).unwrap();
        arr.set(0, Int64(2));
        arr.set(1, Int64(3));
        arr.set(2, Int64(5));
        arr.set(3, Int64(7));
        arr.set(4, Int64(11));
        arr.set(5, Int64(13));
        assert_eq!(arr.binarysearch(Int64(4)), NotFound(2));

        let mut v = vec![0x91];
        for _ in 0..1 {
            v.push(0xd3);
            for _ in 0..8 {
                v.push(0);
            }
        }
        let mut arr = MsgpackArray::parse(v).unwrap();
        arr.set(0, Int64(7));
        assert_eq!(arr.binarysearch(Int64(3)), NotFound(0));
        assert_eq!(arr.binarysearch(Int64(8)), NotFound(1));

        let arr = MsgpackArray::parse(vec![0x90]).unwrap();
        assert_eq!(arr.binarysearch(Int64(8)), NotFound(0));
    }

    #[test]
    fn test_binarysearch_first() {
        let mut array: MsgpackArray<Vec<u8>, Int64> =
            MsgpackArray::new(|len| Ok(vec![0u8; len])).unwrap();
        assert_eq!(array.binarysearch(Int64(3)), NotFound(0));

        array.insert_at(0, Int64(3));
        assert_eq!(array.binarysearch(Int64(2)), NotFound(0));

        array.insert_at(0, Int64(2));
        assert_eq!(array.binarysearch(Int64(1)), NotFound(0));

        array.insert_at(0, Int64(1));
        assert_eq!(array.binarysearch(Int64(1)), Found(0));
    }

    #[test]
    fn test_binarysearch_last() {
        let mut array: MsgpackArray<Vec<u8>, Int64> =
            MsgpackArray::new(|len| Ok(vec![0u8; len])).unwrap();
        assert_eq!(array.binarysearch(Int64(1)), NotFound(0));

        array.insert_at(0, Int64(1));
        assert_eq!(array.binarysearch(Int64(2)), NotFound(1));

        array.insert_at(1, Int64(2));
        assert_eq!(array.binarysearch(Int64(3)), NotFound(2));

        array.insert_at(2, Int64(3));
        assert_eq!(array.binarysearch(Int64(3)), Found(2));
    }

    #[test]
    #[ignore]
    fn test_insert_at() {
        let mut array: MsgpackArray<Vec<u8>, Int64> =
            MsgpackArray::new(|len| Ok(vec![0u8; len])).unwrap();
        array.insert_at(0, Int64(2));
        assert_eq!(array.header(), ArrayHeader::Fix(1));

        for (i, n) in [3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
            .iter()
            .enumerate()
        {
            array.insert_at(i + 1, Int64(*n));
        }
        assert_eq!(array.header(), ArrayHeader::Fix(15));
        assert_eq!(array.get(14), Some(Int64(47)));

        array.insert_at(15, Int64(53));
        assert_eq!(array.header(), ArrayHeader::Array16(16));
        assert_eq!(array.get(0), Some(Int64(2)));
        assert_eq!(array.get(5), Some(Int64(13)));
        assert_eq!(array.get(15), Some(Int64(53)));

        for i in 0..65519 {
            array.insert_at(16 + i, Int64(i as i64));
        }
        assert_eq!(array.header(), ArrayHeader::Array16(65535));
        assert_eq!(array.get(65534), Some(Int64(65518)));

        array.insert_at(32768, Int64(-42));
        assert_eq!(array.header(), ArrayHeader::Array32(65536));
        assert_eq!(array.get(0), Some(Int64(2)));
        assert_eq!(array.get(32768), Some(Int64(-42)));
        assert_eq!(array.get(65535), Some(Int64(65518)));
    }

    #[test]
    fn test_delete_at() {
        let mut array: MsgpackArray<Vec<u8>, Int64> =
            MsgpackArray::new(|len| Ok(vec![0u8; len])).unwrap();
        for (i, n) in [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59,
        ]
        .iter()
        .enumerate()
        {
            array.insert_at(i, Int64(*n));
        }
        assert_eq!(array.header(), ArrayHeader::Array16(17));

        array.delete_at(0);
        assert_eq!(array.header(), ArrayHeader::Array16(16));
        assert_eq!(array.get(0), Some(Int64(3)));
        assert_eq!(array.get(15), Some(Int64(59)));

        array.delete_at(15);
        assert_eq!(array.header(), ArrayHeader::Fix(15));
        assert_eq!(array.get(0), Some(Int64(3)));
        assert_eq!(array.get(14), Some(Int64(53)));

        array.delete_at(5);
        assert_eq!(array.header(), ArrayHeader::Fix(14));
        assert_eq!(array.get(0), Some(Int64(3)));
        assert_eq!(array.get(13), Some(Int64(53)));
        assert_eq!(array.get(5), Some(Int64(19)));

        array.delete_at(99);
        assert_eq!(array.header(), ArrayHeader::Fix(14));
    }
}
