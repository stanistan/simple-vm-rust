FROM rustlang/rust:nightly

ENV BASE_DIR=/simple_vm \
    BENCH_DIR=/simple_vm/bench \
    VOLUME_DIR=/prof \
    VOLUME_DATA_DIR=/prof/data

# To get `perf`
RUN apt-get update && apt-get install -y linux-perf && \
        apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* && \
        mv /usr/bin/perf_* /usr/bin/perf

WORKDIR $BASE_DIR

# This should end up all in getting all of the prod dependencies form
# `simple_vm` built separately from the rest of the lib so changes to
# the local code won't trigger full rebuilds.
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN mkdir src && echo "fn hi() { }" > src/lib.rs && \
        echo "fn main() { }" > src/bin.rs
RUN cargo build --release

# Now we can build/test the rest of it
COPY src src
COPY examples examples
RUN cargo build --release
RUN cargo test --release

# Do the same for the tests
WORKDIR $BENCH_DIR
RUN mkdir src && echo "fn main() { }" > src/main.rs
COPY bench/Cargo.toml Cargo.toml
COPY bench/Cargo.lock Cargo.lock
RUN cargo build --release

# Drop in the actual bench src
COPY bench/ .
RUN cargo bench --no-run

VOLUME $VOLUME_DIR
