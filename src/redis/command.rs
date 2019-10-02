//! Redis commands implementation.

use super::*;
use crate::msgpack::format::Int64;
use crate::msgpack::MsgpackArray;
use crate::msgpack::SearchResult;
use dma::RedisDMA;
use libc::{c_int, size_t};

/// Upsert int64 to array32
///
/// `redis-cli> MSGPACK.UPSERTI64 key [element ...]`
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn UpsertI64_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int,
) -> c_int {
    unsafe {
        RedisModule_AutoMemory(ctx);

        if argc < 2 {
            return RedisModule_WrongArity(ctx);
        }

        let Key(key, key_type) = open_rw(ctx, *argv.add(1));

        if key_type != REDISMODULE_KEYTYPE_EMPTY && key_type != REDISMODULE_KEYTYPE_STRING {
            return reply_wrong_type(ctx);
        }

        let mut array: MsgpackArray<RedisDMA, Int64>;
        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            array = match MsgpackArray::new(|len| {
                let ret = RedisModule_StringTruncate(key, len);
                if ret != REDISMODULE_OK {
                    Err(ret)
                } else {
                    Ok(string_dma(key))
                }
            }) {
                Ok(arr) => arr,
                Err(err) => return err,
            };
        } else {
            array = match MsgpackArray::parse(string_dma(key)) {
                None => return reply_wrong_type(ctx),
                Some(arr) => arr,
            }
        }

        let mut updated_count = 0;
        for i in 2..argc {
            let mut ll = 0;
            if RedisModule_StringToLongLong(*argv.add(i as usize), &mut ll) != REDISMODULE_OK {
                return REDISMODULE_ERR;
            }

            let idx_to_insert = match array.binarysearch(Int64(ll)) {
                SearchResult::Found(_) => continue,
                SearchResult::NotFound(idx) => idx,
            };

            match array.insert_at(idx_to_insert, Int64(ll)) {
                Err(err) => return err,
                _ => {}
            };

            updated_count += 1;
        }

        if updated_count > 0 {
            RedisModule_ReplicateVerbatim(ctx);
        }

        RedisModule_ReplyWithLongLong(ctx, if updated_count > 0 { 1 } else { 0 })
    }
}

/// Delete int64 from array32
///
/// `redis-cli> MSGPACK.DELI64 key [element ...]`
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn DelI64_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int,
) -> c_int {
    unsafe {
        RedisModule_AutoMemory(ctx);

        if argc < 2 {
            return RedisModule_WrongArity(ctx);
        }

        let Key(key, key_type) = open_rw(ctx, *argv.add(1));

        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            return RedisModule_ReplyWithLongLong(ctx, 0);
        }

        if key_type != REDISMODULE_KEYTYPE_STRING {
            return reply_wrong_type(ctx);
        }

        let mut array: MsgpackArray<RedisDMA, Int64> = match MsgpackArray::parse(string_dma(key)) {
            None => return reply_wrong_type(ctx),
            Some(arr) => arr,
        };

        let mut deleted_count = 0;
        for i in 2..argc {
            let mut ll = 0;
            if RedisModule_StringToLongLong(*argv.add(i as usize), &mut ll) != REDISMODULE_OK {
                return REDISMODULE_ERR;
            }

            let idx_to_delete = match array.binarysearch(Int64(ll)) {
                SearchResult::NotFound(_) => continue,
                SearchResult::Found(idx) => idx,
            };

            match array.delete_at(idx_to_delete) {
                Err(err) => return err,
                _ => {}
            };

            deleted_count += 1;
        }

        if deleted_count > 0 {
            RedisModule_ReplicateVerbatim(ctx);
        }

        RedisModule_ReplyWithLongLong(ctx, deleted_count)
    }
}

struct Key(*mut RedisModuleKey, c_int);

fn open_rw(ctx: *mut RedisModuleCtx, string: *mut RedisModuleString) -> Key {
    unsafe {
        let ptr = RedisModule_OpenKey(ctx, string, REDISMODULE_READ | REDISMODULE_WRITE);
        let key_type = RedisModule_KeyType(ptr);

        Key(ptr, key_type)
    }
}

fn string_dma(key: *mut RedisModuleKey) -> RedisDMA {
    let mut len: size_t = 0;
    unsafe {
        let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);
        RedisDMA::wrap(key, ptr, len)
    }
}

fn reply_wrong_type(ctx: *mut RedisModuleCtx) -> c_int {
    unsafe {
        RedisModule_ReplyWithError(
            ctx,
            "WRONGTYPE Key is not a valid msgpack string value.\0".as_ptr(),
        )
    }
}
