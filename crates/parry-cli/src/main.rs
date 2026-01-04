use clap::{Args, Parser, Subcommand};
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::ErrorKind;
use std::path::PathBuf;

use parry::ReedSolomonEncoder;

#[derive(Parser, Debug)]
#[command(
    name = "parry-cli",
    version,
    about = "Tool to encode and decode files using Reed-Solomon encoding"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Encode(EncodeArgs),
    Decode(DecodeArgs),
}

#[derive(Args, Debug, Clone)]
struct CommonArgs {
    #[arg(long, value_name = "N")]
    data_shards: usize,

    #[arg(long, value_name = "N")]
    parity_shards: usize,

    #[arg(long, value_name = "BYTES")]
    chunk_size: usize,
}

#[derive(Args, Debug)]
struct EncodeArgs {
    #[command(flatten)]
    common: CommonArgs,

    #[arg(long, value_name = "FILE")]
    data_file: PathBuf,

    #[arg(long, value_name = "PATTERN")]
    shard_file_pattern: String,
}

#[derive(Args, Debug)]
struct DecodeArgs {
    #[command(flatten)]
    common: CommonArgs,

    #[arg(long, value_name = "PATTERN")]
    shard_file_pattern: String,

    #[arg(long, value_name = "FILE")]
    data_file: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Encode(args) => {
            let encoder = ReedSolomonEncoder::new(
                args.common.data_shards,
                args.common.parity_shards,
                args.common.chunk_size,
            );

            let data_file = File::open(args.data_file).unwrap();
            let length = data_file.metadata().unwrap().len() as usize;
            let mut buffered_input_file = BufReader::new(data_file);

            let mut output_files =
                Vec::with_capacity(args.common.data_shards + args.common.parity_shards);
            for shard in 0..args.common.data_shards + args.common.parity_shards {
                output_files.push(BufWriter::new(
                    File::create(args.shard_file_pattern.replace("{}", &shard.to_string()))
                        .unwrap(),
                ));
            }

            encoder
                .encode(&mut buffered_input_file, length, &mut output_files)
                .unwrap();
        }
        Command::Decode(args) => {
            let encoder = ReedSolomonEncoder::new(
                args.common.data_shards,
                args.common.parity_shards,
                args.common.chunk_size,
            );

            let mut input_files = (0..(args.common.data_shards + args.common.parity_shards))
                .map(|shard| {
                    let shard_path = args.shard_file_pattern.replace("{}", &shard.to_string());

                    match File::open(shard_path) {
                        Ok(shard_file) => Some(BufReader::new(shard_file)),
                        Err(io_error) => {
                            if io_error.kind() == ErrorKind::NotFound {
                                None
                            } else {
                                panic!("FIXME: unhandle IO exception");
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();

            let mut data_file = BufWriter::new(File::create(args.data_file).unwrap());

            encoder
                .decode(input_files.as_mut_slice(), &mut data_file)
                .unwrap();
        }
    }
}
