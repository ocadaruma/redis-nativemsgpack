//! Redis commands implementation.

use super::*;
use dma::CByteArray;
use libc::{c_double, c_int, size_t, c_longlong};
use std::slice::from_raw_parts;
use crate::msgpack::{MsgpackArray, ByteVector};
use crate::msgpack::SearchResult;

impl ByteVector for CByteArray {
    fn len(&self) -> usize {
        CByteArray::len(self)
    }

    fn memmove(&mut self, dest: usize, src: usize, len: usize) {
        unimplemented!()
    }

    fn realloc(&self, len: usize) -> Self {
        unimplemented!()
    }

    fn alloc(len: usize) -> Self {
        unimplemented!()
    }
}

/// Upsert int64 to array32
///
/// `redis-cli> MSGPACK.UPSERTI64 key [element ...]`
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn UpsertI64_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int {

//    unsafe {
//        RedisModule_AutoMemory(ctx);
//
//        if argc < 2 {
//            return RedisModule_WrongArity(ctx);
//        }
//
//        let Key(key, key_type) = open_rw(ctx, *argv.add(1));
//
//        if key_type != REDISMODULE_KEYTYPE_EMPTY && key_type != REDISMODULE_KEYTYPE_STRING {
//            return reply_wrong_type(ctx);
//        }
//        if key_type == REDISMODULE_KEYTYPE_EMPTY {
//            if RedisModule_StringTruncate(key, 5) != REDISMODULE_OK {
//                return REDISMODULE_ERR;
//            }
//            MsgpackArray::initialize(string_dma(key));
//        }
//
//        let current_arr = match MsgpackArray::parse(string_dma(key)) {
//            None => return reply_wrong_type(ctx),
//            Some(arr) => arr
//        };
//
//        let mut updated_count = 0;
//        for i in 2..argc {
//            let mut ll  = 0;
//            if RedisModule_StringToLongLong(*argv.add(i as usize), &mut ll) != REDISMODULE_OK {
//                return REDISMODULE_ERR;
//            }
//
//            let idx_to_insert = match current_arr.binarysearch(ll) {
//                SearchResult::Found(_) => continue,
//                SearchResult::NotFound(idx) => idx
//            };
//
//            if RedisModule_StringTruncate(key, (current_arr.len + 1) * 9 + 5) != REDISMODULE_OK {
//                return REDISMODULE_ERR;
//            }
//
//            let mut len: size_t = 0;
//            let mut ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);
//
//            if current_arr.len < idx_to_insert {
//            } else {
//                ptr.add(idx_to_insert * 9 + 5).copy_to(ptr.add((idx_to_insert + 1) * 9 + 5),
//                                                       (current_arr.len - idx_to_insert) * 9);
//                let mut carr = CByteArray::wrap(ptr, len);
//
//                let n = current_arr.len + 1;
//                for i in 0..4 {
//                    carr[1 + i] = ((n >> (i * 4)) & 0xff) as u8;
//                }
//            }
//            MsgpackArray::parse(CByteArray::wrap(ptr, len)).unwrap().set(idx_to_insert, ll);
//
//            updated_count += 1;
//        }
//
//        if updated_count > 0 {
//            RedisModule_ReplicateVerbatim(ctx);
//        }
//
//        RedisModule_ReplyWithLongLong(ctx, if updated_count > 0 { 1 } else { 0 })
//    }

    unimplemented!()
}

/// Delete int64 from array32
///
/// `redis-cli> MSGPACK.DELI64 key [element ...]`
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn DelI64_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int {

    unimplemented!()
}

struct Key(*mut RedisModuleKey, c_int);

fn open_ro(ctx: *mut RedisModuleCtx, string: *mut RedisModuleString) -> Key {
    unsafe {
        let ptr = RedisModule_OpenKey(ctx, string, REDISMODULE_READ);
        let key_type = RedisModule_KeyType(ptr);

        Key(ptr, key_type)
    }
}

fn open_rw(ctx: *mut RedisModuleCtx, string: *mut RedisModuleString) -> Key {
    unsafe {
        let ptr = RedisModule_OpenKey(ctx, string, REDISMODULE_READ | REDISMODULE_WRITE);
        let key_type = RedisModule_KeyType(ptr);

        Key(ptr, key_type)
    }
}

fn string_dma(key: *mut RedisModuleKey) -> CByteArray {
    let mut len: size_t = 0;
    unsafe {
        let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);
        CByteArray::wrap(ptr, len)
    }
}

fn reply_wrong_type(ctx: *mut RedisModuleCtx) -> c_int {
    unsafe {
        RedisModule_ReplyWithError(
            ctx, "WRONGTYPE Key is not a valid msgpack string value.\0".as_ptr())
    }
}

fn reply_ok(ctx: *mut RedisModuleCtx) -> c_int {
    unsafe {
        RedisModule_ReplyWithSimpleString(ctx, "OK\0".as_ptr())
    }
}
