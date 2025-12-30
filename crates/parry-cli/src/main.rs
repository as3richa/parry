use clap::{Args, Parser, Subcommand};
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
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
    input_file: PathBuf,

    #[arg(long, value_name = "PATTERN")]
    output_file_pattern: String,
}

#[derive(Args, Debug)]
struct DecodeArgs {
    #[command(flatten)]
    common: CommonArgs,

    #[arg(long, value_name = "PATTERN")]
    input_file_pattern: String,

    #[arg(long, value_name = "FILE")]
    output_file: PathBuf,
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

            let input_file = File::open(args.input_file).unwrap();
            let length = input_file.metadata().unwrap().len() as usize;
            let mut buffered_input_file = BufReader::new(input_file);

            let mut output_files =
                Vec::with_capacity(args.common.data_shards + args.common.parity_shards);
            for shard in 0..args.common.data_shards + args.common.parity_shards {
                output_files.push(BufWriter::new(
                    File::create(args.output_file_pattern.replace("{}", &shard.to_string()))
                        .unwrap(),
                ));
            }

            encoder
                .encode(&mut buffered_input_file, length, &mut output_files)
                .unwrap();
        }
        Command::Decode(args) => {
            eprintln!(
                "decode: data_shards={}, parity_shards={}, chunk_size={}, input_file_pattern={}, output_file={:?}",
                args.common.data_shards,
                args.common.parity_shards,
                args.common.chunk_size,
                args.input_file_pattern,
                args.output_file
            );
        }
    }
}
