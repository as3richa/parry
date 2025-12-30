mod field;
mod gf8;
mod io;
mod matrix;

use std::io::{Read, Seek, Write};
use std::slice;
use xxhash_rust::xxh3::xxh3_128;

use crate::gf8::Gf8;
use crate::io::{SeekableShardReader, ShardReadError, ShardReader};
use crate::matrix::Matrix;

pub struct ReedSolomonEncoder {
    data_shards: usize,
    parity_shards: usize,
    chunk_size: usize,
}

impl ReedSolomonEncoder {
    pub fn new(data_shards: usize, parity_shards: usize, chunk_size: usize) -> ReedSolomonEncoder {
        assert!(
            data_shards + parity_shards <= 256,
            "Total number of shards cannot exceed 256"
        );

        assert!(chunk_size > 0, "Chunk size must be greater than zero");

        ReedSolomonEncoder {
            data_shards,
            parity_shards,
            chunk_size,
        }
    }

    pub fn encode<R: Read, W: Write>(
        &self,
        data: &mut R,
        length: usize,
        output: &mut [W],
    ) -> std::io::Result<()> {
        assert_eq!(output.len(), self.data_shards + self.parity_shards);

        let encoding_matrix = Matrix::<Gf8>::encoding_matrix(self.data_shards, self.parity_shards);
        let encoding_matrix_excluding_identity =
            encoding_matrix.slice(self.data_shards..self.data_shards + self.parity_shards);

        let block_size = self.data_shards * self.chunk_size;
        let loops = (length + 8).div_ceil(block_size);
        let final_loop_block_size = if (length + 8).is_multiple_of(block_size) {
            block_size
        } else {
            (length + 8) % block_size
        };

        let mut data_matrix = Matrix::<Gf8>::with_dimensions(self.data_shards, self.chunk_size);

        for i in 0..loops {
            {
                let mut buffer: &mut [u8] = unsafe {
                    slice::from_raw_parts_mut(data_matrix.elements.as_ptr() as *mut u8, block_size)
                };

                if i == 0 {
                    let encoded_length = (length as u64).to_be_bytes();
                    buffer[0..8].copy_from_slice(&encoded_length);
                    data.read_exact(&mut buffer[8..block_size])?;
                } else if i == loops - 1 {
                    data.read_exact(&mut buffer[0..final_loop_block_size])?;
                    buffer[final_loop_block_size..block_size].fill(0);
                } else {
                    data.read_exact(&mut buffer)?;
                }

                for shard in 0..self.data_shards {
                    let data = &buffer[shard * self.chunk_size..(shard + 1) * self.chunk_size];
                    output[shard].write_all(&xxh3_128(data).to_be_bytes())?;
                    output[shard].write_all(data)?;
                }
            }
            // FIXME: ownership?????
            let encoded_data_matrix = &encoding_matrix_excluding_identity * &data_matrix;

            {
                let buffer: &[u8] = unsafe {
                    slice::from_raw_parts(
                        encoded_data_matrix.elements.as_ptr() as *mut u8,
                        self.parity_shards * self.chunk_size,
                    )
                };

                for parity_shard in 0..self.parity_shards {
                    let shard = self.data_shards + parity_shard;
                    let data = &buffer
                        [parity_shard * self.chunk_size..(parity_shard + 1) * self.chunk_size];
                    output[shard].write_all(&xxh3_128(data).to_be_bytes())?;
                    output[shard].write_all(data)?;
                }
            }
        }

        Result::Ok(())
    }

    pub fn decode<R: ShardReader, W: Write>(
        data: &[Option<R>],
        output: W,
    ) -> Result<(), ShardReadError> {
        Result::Ok(())
    }

    pub fn decode_at<R: SeekableShardReader, W: Write>(
        data: &[Option<R>],
        output: W,
        offset: usize,
        length: usize,
    ) -> Result<(), ShardReadError> {
        Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};
    use std::io::Cursor;

    #[test]
    fn encode() {
        let mut rng = StdRng::from_seed([42u8; 32]);

        let mut buffer = vec![0u8; 16 * 1024];
        rng.fill_bytes(&mut buffer);

        let mut reader = Cursor::new(buffer);

        let mut writers: Vec<Cursor<Vec<u8>>> =
            (0..6).map(|_| Cursor::new(Vec::<u8>::new())).collect();

        ReedSolomonEncoder::new(4, 2, 1024)
            .encode(&mut reader, 16 * 1024, &mut writers)
            .unwrap();
    }
}
