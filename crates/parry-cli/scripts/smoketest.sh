#!/usr/bin/env bash

set -euo pipefail

DATA_SHARDS=17
PARITY_SHARDS=3
CHUNK_SIZE=200000

ORIGINAL_DATA_FILE="data-file"
RESTORED_DATA_FILE="restored-data-file"
SHARD_FILE_PATTERN_PREFIX="data-file-shard-"

COMMON_FLAGS="--data-shards $DATA_SHARDS --parity-shards $PARITY_SHARDS --chunk-size $CHUNK_SIZE --shard-file-pattern $SHARD_FILE_PATTERN_PREFIX{}"

BINARY="target/release/parry-cli"
ENCODE_COMMAND="$BINARY encode $COMMON_FLAGS --data-file $ORIGINAL_DATA_FILE"
DECODE_COMMAND="$BINARY decode $COMMON_FLAGS --data-file $RESTORED_DATA_FILE"
CHECKSUM_COMMAND="shasum $ORIGINAL_DATA_FILE $RESTORED_DATA_FILE"
CLEANUP_COMMAND="rm $RESTORED_DATA_FILE $SHARD_FILE_PATTERN_PREFIX*"

cargo build --release

dd if=/dev/urandom of=$ORIGINAL_DATA_FILE bs=$((100*1024*1024)) count=1

echo "=> With all shards present"
$ENCODE_COMMAND
$DECODE_COMMAND
$CHECKSUM_COMMAND
$CLEANUP_COMMAND

for SHARD1 in $(seq 0 $(($DATA_SHARDS + $PARITY_SHARDS - 1))); do
    for SHARD2 in $(seq 0 $(($DATA_SHARDS + $PARITY_SHARDS - 1))); do
        echo "=> With shard(s) $SHARD1 and $SHARD2 missing"
        time $ENCODE_COMMAND
        rm "$SHARD_FILE_PATTERN_PREFIX$SHARD1"
        if [[ $SHARD1 != $SHARD2 ]]; then
            rm "$SHARD_FILE_PATTERN_PREFIX$SHARD2"
        fi
        time $DECODE_COMMAND
        $CHECKSUM_COMMAND
        $CLEANUP_COMMAND
    done
done

rm $ORIGINAL_DATA_FILE