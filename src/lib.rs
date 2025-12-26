mod field;
mod gf8;
mod io;
mod matrix;

use std::io::{Error, Read, Seek, Write};

pub struct ReedSolomonEncoder {
    data_shards: usize,
    parity_shards: usize,
    shard_size: usize,
}

impl ReedSolomonEncoder {
    pub fn new(data_shards: usize, parity_shards: usize, shard_size: usize) -> ReedSolomonEncoder {
        ReedSolomonEncoder {
            data_shards,
            parity_shards,
            shard_size,
        }
    }

    pub fn encode<R: Read, W: Write>(data: R, length: usize, output: &[W]) -> Result<(), Error> {
        Result::Ok(())
    }

    pub fn decode<R: Read, W: Write>(data: &[Option<R>], output: W) -> Result<(), Error> {
        Result::Ok(())
    }

    pub fn decode_at<R: Read + Seek, W: Write>(
        data: &[Option<R>],
        output: W,
        offset: usize,
        length: usize,
    ) -> Result<(), Error> {
        Result::Ok(())
    }
}
