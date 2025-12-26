use std::io;

trait ShardReader {
    fn read_chunk(data: &mut [u8], offset: usize) -> io::Result<()>;
}

trait ShardWriter {
    fn write_chunk(data: &[u8]) -> io::Result<()>;
}
