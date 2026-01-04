mod field;
mod gf8;
mod io;
mod matrix;

use num_traits::PrimInt;
use num_traits::Unsigned;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Read, Seek, Write};
use std::iter;
use std::slice;

use crate::gf8::Gf8;
use crate::io::{ChunkReadError, read_chunk, write_chunk};
use crate::matrix::Matrix;

pub struct ReedSolomonEncoder {
    data_shards: usize,
    parity_shards: usize,
    chunk_size: usize,
}

#[derive(Debug)]
pub enum DecodeError {
    IoError(std::io::Error),
    Truncated,
    ChecksumValidationFailure,
    InsufficientShards,
}

trait ShardSet: Eq + Hash + Clone {
    fn max_shards() -> usize;

    fn empty() -> Self;

    fn insert(&mut self, shard: usize);

    fn elements(&self) -> impl Iterator<Item = usize>;
}

trait SmallShardSetStorage: Unsigned + PrimInt + Hash {}

impl<T: Unsigned + PrimInt + Hash> SmallShardSetStorage for T {}

#[derive(PartialEq, Eq, Hash, Clone)]
struct SmallShardSet<T: SmallShardSetStorage>(T);

impl<T: SmallShardSetStorage> ShardSet for SmallShardSet<T> {
    fn max_shards() -> usize {
        T::zero().count_zeros() as usize
    }

    fn empty() -> Self {
        SmallShardSet(T::zero())
    }

    fn insert(&mut self, shard: usize) {
        self.0 = self.0 | (T::one() << shard);
    }

    fn elements(&self) -> impl Iterator<Item = usize> {
        let mut value = self.0;

        iter::from_fn(move || {
            if value.is_zero() {
                return None;
            }

            let trailing_zeroes = value.trailing_zeros() as usize;
            value = value ^ (T::one() << trailing_zeroes);

            Some(trailing_zeroes)
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct BigShardSet<const N: usize>([u64; N]);

impl<const N: usize> ShardSet for BigShardSet<N> {
    fn max_shards() -> usize {
        N * 64
    }

    fn empty() -> Self {
        Self([0u64; N])
    }

    fn insert(&mut self, shard: usize) {
        self.0[shard / 64] |= 1 << (shard % 64);
    }

    fn elements(&self) -> impl Iterator<Item = usize> {
        self.0.iter().enumerate().flat_map(|(i, value)| {
            let mut value = *value;
            let offset = i * 64;

            iter::from_fn(move || {
                if value == 0 {
                    return None;
                }

                let trailing_zeroes = value.trailing_zeros() as usize;
                value ^= 1 << trailing_zeroes;

                Some(trailing_zeroes + offset)
            })
        })
    }
}

impl ReedSolomonEncoder {
    pub fn new(data_shards: usize, parity_shards: usize, chunk_size: usize) -> ReedSolomonEncoder {
        assert!(
            data_shards + parity_shards <= 256,
            "Total number of shards cannot exceed 256"
        );

        assert!(chunk_size > 8, "Chunk size must be at least 8");

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
        shard_writers: &mut [W],
    ) -> std::io::Result<()> {
        assert_eq!(shard_writers.len(), self.data_shards + self.parity_shards);

        let encoding_matrix = Matrix::<Gf8>::encoding_matrix(self.data_shards, self.parity_shards);
        let encoding_matrix_excluding_identity =
            encoding_matrix.slice(self.data_shards..self.data_shards + self.parity_shards);

        let block_size = self.data_shards * self.chunk_size;
        let blocks = (length + 8).div_ceil(block_size);
        let final_loop_block_size = if (length + 8).is_multiple_of(block_size) {
            block_size
        } else {
            (length + 8) % block_size
        };

        let mut data_matrix = Matrix::<Gf8>::with_dimensions(self.data_shards, self.chunk_size);
        let mut encoded_parity_shard_matrix = Matrix::<Gf8>::with_dimensions(self.parity_shards, self.chunk_size);

        for i in 0..blocks {
            {
                let buffer: &mut [u8] = unsafe {
                    slice::from_raw_parts_mut(
                        data_matrix.elements.as_mut_ptr() as *mut u8,
                        block_size,
                    )
                };

                if i == 0 {
                    let encoded_length = (length as u64).to_be_bytes();
                    buffer[0..8].copy_from_slice(&encoded_length);
                    data.read_exact(&mut buffer[8..block_size])?;
                } else if i == blocks - 1 {
                    data.read_exact(&mut buffer[0..final_loop_block_size])?;
                    buffer[final_loop_block_size..block_size].fill(0);
                } else {
                    data.read_exact(buffer)?;
                }

                for shard in 0..self.data_shards {
                    let chunk = &buffer[shard * self.chunk_size..(shard + 1) * self.chunk_size];
                    write_chunk(&mut shard_writers[shard], chunk)?;
                }
            }

            encoded_parity_shard_matrix.multiply_fast_in_place_with_multiplication_table(&encoding_matrix_excluding_identity, &data_matrix);

            {
                let buffer: &[u8] = unsafe {
                    slice::from_raw_parts(
                        encoded_parity_shard_matrix.elements.as_ptr() as *mut u8,
                        self.parity_shards * self.chunk_size,
                    )
                };

                for parity_shard in 0..self.parity_shards {
                    let shard = self.data_shards + parity_shard;
                    let chunk = &buffer
                        [parity_shard * self.chunk_size..(parity_shard + 1) * self.chunk_size];
                    write_chunk(&mut shard_writers[shard], chunk)?;
                }
            }
        }

        Result::Ok(())
    }

    pub fn decode<R: Read, W: Write>(
        &self,
        shards: &mut [Option<R>],
        output: &mut W,
    ) -> Result<(), DecodeError> {
        let total_shards = self.data_shards + self.parity_shards;

        macro_rules! shard_set_branch {
            ($type: ty) => {
                if total_shards <= <$type>::max_shards() {
                    return self.decode_at_with_shard_set::<R, W, $type>(shards, output, None);
                }
            };
        }

        assert!(total_shards <= BigShardSet::<4>::max_shards());

        shard_set_branch!(SmallShardSet<u8>);
        shard_set_branch!(SmallShardSet<u16>);
        shard_set_branch!(SmallShardSet<u32>);
        shard_set_branch!(SmallShardSet<u64>);
        shard_set_branch!(BigShardSet<4>);
        unreachable!();
    }

    pub fn decode_at<R: Read + Seek, W: Write>(
        _data: &mut [Option<R>],
        _output: &mut W,
        _offset: usize,
        _length: usize,
    ) -> std::io::Result<()> {
        // FIXME
        Result::Ok(())
    }

    fn decode_at_with_shard_set<R: Read, W: Write, S: ShardSet>(
        &self,
        shards: &mut [Option<R>],
        output: &mut W,
        offset_and_length: Option<(usize, usize)>,
    ) -> Result<(), DecodeError> {
        assert_eq!(shards.len(), self.data_shards + self.parity_shards);

        let mut present_shards = shards
            .iter_mut()
            .enumerate()
            .filter_map(|(i, shard)| shard.as_mut().map(|shard| (i, shard)))
            .collect::<Vec<(usize, &mut R)>>();

        if present_shards.len() < self.data_shards {
            return Result::Err(DecodeError::InsufficientShards);
        }

        assert!(offset_and_length.is_none()); // FIXME

        let encoding_matrix = Matrix::<Gf8>::encoding_matrix(self.data_shards, self.parity_shards);
        let mut decoding_matrix_for_shard_set = HashMap::<S, Matrix<Gf8>>::new();

        let mut encoded_data_matrix =
            Matrix::<Gf8>::with_dimensions(self.data_shards, self.chunk_size);

        let mut block = 0usize;
        let mut blocks = 1usize;

        let mut final_loop_block_size = 0usize; // FIXME: insane data modeling?

        let mut data_matrix = Matrix::with_dimensions(self.data_shards, self.chunk_size);

        while block < blocks {
            let buffer: &mut [u8] = unsafe {
                slice::from_raw_parts_mut(
                    encoded_data_matrix.elements.as_mut_ptr() as *mut u8,
                    self.data_shards * self.chunk_size,
                )
            };

            let mut shard_set = S::empty();
            let mut chunks_read = 0;

            for (shard, shard_data) in present_shards.iter_mut() {
                if chunks_read >= self.data_shards {
                    break;
                }

                let chunk =
                    &mut buffer[chunks_read * self.chunk_size..(chunks_read + 1) * self.chunk_size];

                match read_chunk(*shard_data, chunk) {
                    Err(ChunkReadError::IoError(io_error)) => {
                        return Err(DecodeError::IoError(io_error));
                    }
                    Err(ChunkReadError::ChecksumValidationFailure)
                    | Err(ChunkReadError::Truncated) => {}
                    Ok(()) => {
                        shard_set.insert(*shard);
                        chunks_read += 1;
                    }
                }
            }

            if chunks_read < self.data_shards {
                // FIXME: error modeling? Propagate the chunk read error? Which one?
                return Result::Err(DecodeError::InsufficientShards);
            }

            assert_eq!(chunks_read, self.data_shards);

            let decoding_matrix = decoding_matrix_for_shard_set
                .entry(shard_set.clone())
                .or_insert_with(|| {
                    let mut partial_encoding_matrix =
                        Matrix::with_dimensions(self.data_shards, self.data_shards);

                    for (row, shard) in shard_set.elements().enumerate() {
                        partial_encoding_matrix[row].copy_from_slice(&encoding_matrix[shard]);
                    }

                    partial_encoding_matrix.invert().unwrap()
                });

            data_matrix.multiply_fast_in_place_with_multiplication_table(&*decoding_matrix, &encoded_data_matrix);

            let block_size = self.data_shards * self.chunk_size;
            let buffer = unsafe {
                slice::from_raw_parts(data_matrix.elements.as_ptr() as *const u8, block_size)
            };

            let to_write = if block == 0 {
                let length = u64::from_be_bytes(buffer[0..8].try_into().unwrap()) as usize;
                blocks = (length + 8).div_ceil(block_size);
                final_loop_block_size = if (length + 8).is_multiple_of(block_size) {
                    block_size
                } else {
                    (length + 8) % block_size
                };
                &buffer[8..block_size]
            } else if block == blocks - 1 {
                &buffer[0..final_loop_block_size]
            } else {
                buffer
            };

            output.write_all(to_write).map_err(DecodeError::IoError)?;

            block += 1;
        }

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
