FROM rustlang/rust:nightly

ENV BASE_DIR=/usr/src/simple_vm \
    VOLUME_DIR=/prof \
    VOLUME_DATA_DIR=/prof/data

# To get `perf`
RUN apt-get update && apt-get install -y linux-perf && \
        apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* && \
        mv /usr/bin/perf_* /usr/bin/perf

WORKDIR /usr/src
RUN USER=root cargo new simple_vm --lib
COPY Cargo.toml Cargo.lock $BASE_DIR/

WORKDIR $BASE_DIR
RUN cargo build --release
RUN rm src/*.rs

COPY src $BASE_DIR/src
COPY examples $BASE_DIR/examples
COPY benches $BASE_DIR/benches

# Build and test to make sure everything
# works correctly before running
# benchmarks
#RUN cargo build --release
#RUN cargo test --release

# Do benchmarks *last*
#RUN cargo bench --no-run

VOLUME $VOLUME_DIR
