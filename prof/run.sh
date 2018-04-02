#!/bin/sh -e
#
# This is meant to be run inside of the docker image defined in `prof/Dockerfile`
#
if [ ! -f /.dockerenv ]; then

    # We are outside of the docker env...
    # let's check if this is the directory with the dockerfile, if so, we can
    # go to the app root and run some stuff :shrug:
    if [ ! -f Cargo.toml ]; then
        echo Not running from the root simple_vm dir, but from `pwd`
        exit 1
    fi

    # Make sure the toml is for the right project
    cat Cargo.toml | grep "name = \"simple_vm\"" > /dev/null || {
        echo Not running from the root simple_vm dir, but from `pwd`
        exit 1
    }

    # build the docker image and run it
    echo
    echo Running outside of a docker context
    echo building the docker image, and will recurse...
    echo
    TAG=simple-vm-prof
    docker build -t $TAG -f prof/Dockerfile .
    docker run --privileged -it -v "`pwd`/prof":/prof $TAG /prof/run.sh "$@"
    echo
    ls -la prof/data
    exit 0
fi

if [ -z "$VOLUME_DATA_DIR" ] || [ -z "$BASE_DIR" ]; then
    echo Missing env vars from Docker image
    exit 1
fi

# We need to find the *raw* executable
# of the benchmarks that we want to run
cd $BENCH_DIR
FILENAME=`find target/release/simple_vm_bench-* -exec file {} \; | grep -i elf | awk -F: '{ print $1 }'`
echo Going to run tests against target file $FILENAME

# Do the weeerk, whatever args are passed into this
# script will be forwarded to the bench arg, so we can
# run it for a particular benchmark
perf record -g $FILENAME --bench "$@" > /dev/null
perf script | grep -v unknown > $VOLUME_DATA_DIR/perf.script
perf report > $VOLUME_DATA_DIR/perf.report

# COOL, we did it!
#
# $VOLUME_DATA_DIR is a volume on the docker image
# so we should be able to run this script mounting a volume
# for writing to be able to get out the out the data
# and use it for flamegraphs/etc

echo
echo DONE
echo
echo
echo Find data in prof/data
