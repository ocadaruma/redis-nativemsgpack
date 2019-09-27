use crate::msgpack::{ByteVector, UnitResult};
use super::*;
use libc::{c_int, size_t};
use std::ops::{Index, IndexMut};

pub struct RedisDMA {
    key: *mut RedisModuleKey,
    underlying: *mut u8,
    len: size_t,
}

impl ByteVector for RedisDMA {
    type AllocErr = c_int;

    fn len(&self) -> usize {
        RedisDMA::len(self)
    }

    fn memmove(&mut self, dest: usize, src: usize, len: usize) {
        unsafe {
            self.underlying.add(src).copy_to(self.underlying.add(dest), len)
        }
    }

    fn realloc(&self, len: usize) -> Result<Self, Self::AllocErr> {
        unsafe {
            let ret = RedisModule_StringTruncate(self.key, len);
            if ret == REDISMODULE_OK {
                let mut len: size_t = 0;
                let ptr = RedisModule_StringDMA(self.key, &mut len, REDISMODULE_WRITE);

                Ok(Self {
                    key: self.key,
                    underlying: ptr,
                    len,
                })
            } else {
                Err(ret)
            }
        }
    }
}

impl RedisDMA {
    pub fn wrap(key: *mut RedisModuleKey, ptr: *mut u8, len: size_t) -> Self {
        RedisDMA { key, underlying: ptr, len, }
    }

    pub fn offset(&self, offset: size_t) -> Self {
        Self::wrap(
            self.key,
            unsafe {
                self.underlying.add(offset)
            },
            self.len - offset)
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Index<usize> for RedisDMA {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &*self.underlying.add(index)
        }
    }
}

impl IndexMut<usize> for RedisDMA {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            &mut *self.underlying.add(index)
        }
    }
}

//#[cfg(test)]
//mod tests {
//    use crate::redis::dma::RedisDMA;
//
//    #[test]
//    fn test_new() {
//        let mut arr = [0u8; 10];
//
//        let dma = RedisDMA::wrap(arr.as_mut_ptr(), arr.len());
//
//        assert_eq!(dma.len, 10);
//    }
//
//    #[test]
//    fn test_index() {
//        let mut arr = [2u8, 3, 5, 7, 11];
//        let mut dma = RedisDMA::wrap(arr.as_mut_ptr(), arr.len());
//
//        assert_eq!(dma.len, 5);
//        for i in 0..arr.len() {
//            assert_eq!(dma[i], arr[i]);
//        }
//
//        dma[3] = 42;
//        assert_eq!(dma[3], 42);
//        assert_eq!(arr[3], 42);
//    }
//}
