use std::io;
use std::io::Read;

pub enum ShardReadError {
    IoError(io::Error),
    DataIntegrityError,
}

impl From<io::Error> for ShardReadError {
    fn from(error: io::Error) -> Self {
        ShardReadError::IoError(error)
    }
}

pub trait ShardReader {
    fn read_chunk(&mut self, data: &mut [u8]) -> Result<(), ShardReadError>;
}

impl<R: Read> ShardReader for R {
    fn read_chunk(&mut self, data: &mut [u8]) -> Result<(), ShardReadError> {
        match self.read_exact(data) {
            Result::Ok(()) => Result::Ok(()),
            Result::Err(error) => {
                if error.kind() == io::ErrorKind::UnexpectedEof {
                    Result::Err(ShardReadError::DataIntegrityError)
                } else {
                    Result::Err(ShardReadError::IoError(error))
                }
            }
        }
    }
}

pub trait SeekableShardReader: ShardReader + io::Seek {}
