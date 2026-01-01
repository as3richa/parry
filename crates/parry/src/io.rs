use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use xxhash_rust::xxh3::xxh3_128;

pub(crate) enum ChunkReadError {
    IoError(io::Error),
    Truncated,
    ChecksumValidationFailure,
}

pub(crate) fn write_chunk<W: Write>(writer: &mut W, chunk: &[u8]) -> io::Result<()> {
    writer.write_all(&xxh3_128(chunk).to_be_bytes())?;
    writer.write_all(chunk)?;
    Result::Ok(())
}

pub(crate) fn read_chunk<R: Read>(reader: &mut R, chunk: &mut [u8]) -> Result<(), ChunkReadError> {
    let mut hash_be_bytes = [0u8; 16];

    reader.read_exact(&mut hash_be_bytes).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            ChunkReadError::Truncated
        } else {
            ChunkReadError::IoError(error)
        }
    })?;

    let hash = u128::from_be_bytes(hash_be_bytes);

    reader.read_exact(chunk).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            ChunkReadError::Truncated
        } else {
            ChunkReadError::IoError(error)
        }
    })?;

    let computed_hash = xxh3_128(chunk);

    if hash == computed_hash {
        Result::Ok(())
    } else {
        Result::Err(ChunkReadError::ChecksumValidationFailure)
    }
}

pub(crate) fn seek_to_chunk<R: Seek>(
    reader: &mut R,
    chunk_number: usize,
    chunk_size: usize,
) -> io::Result<()> {
    reader.seek(SeekFrom::Start((chunk_size as u64) * (chunk_number as u64)))?;
    Result::Ok(())
}
