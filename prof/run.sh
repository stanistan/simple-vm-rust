#!/bin/sh -ex
#
# This totally won't work on MacOS...
# The idea here is to build a release version of the bench mark
# and run it through perf in order to be able to look at the report,
# generate flamegraphs, etc.

BENCH_TARGET="$1"
BENCH_FILTER="$2"

cargo bench --no-run
FILENAME=`find target/release/deps/$BENCH_TARGET-* -exec file {} \; | grep -i elf | awk -F: '{ print $1 }'`

perf record -g $FILENAME --bench "$BENCH_FILTER"
perf script | grep -v unknown > perf.script
perf report > perf.report
