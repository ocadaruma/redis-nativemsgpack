use super::*;
use crate::msgpack::ByteVector;
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
            self.underlying
                .add(src)
                .copy_to(self.underlying.add(dest), len)
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
        RedisDMA {
            key,
            underlying: ptr,
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Index<usize> for RedisDMA {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.underlying.add(index) }
    }
}

impl IndexMut<usize> for RedisDMA {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.underlying.add(index) }
    }
}
