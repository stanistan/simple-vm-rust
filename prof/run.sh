#!/bin/sh -ex
#
# This is meant to be run inside of the docker image defined in `prof/Dockerfile`
if [ -z "$VOLUME_DATA_DIR" ] || [ -z "$BENCH_DIR" ]; then
    echo Missing env vars from Docker image
    exit 1
fi

# We need to find the *raw* executable
# of the benchmarks that we want to run
cd $BENCH_DIR
FILENAME=`find target/release/simple_vm_bench-* -exec file {} \; | grep -i elf | awk -F: '{ print $1 }'`

# Do the weeerk, whatever args are passed into this
# script will be forwarded to the bench arg, so we can
# run it for a particular benchmark
perf record -g $FILENAME --bench "$@"
perf script | grep -v unknown > $VOLUME_DATA_DIR/perf.script
perf report > $VOLUME_DATA_DIR/perf.report

# COOL, we did it!
#
# $VOLUME_DATA_DIR is a volume on the docker image
# so we should be able to run this script mounting a volume
# for writing to be able to get out the out the data
# and use it for flamegraphs/etc
