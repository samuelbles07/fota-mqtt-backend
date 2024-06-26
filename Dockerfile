FROM rust:1.73 as build

RUN USER=root cargo new --bin rocky
WORKDIR rocky 

# Copy deps info and build app depedencies
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs

# delete app bin that previously build and build real app bin 
COPY ./src ./src
RUN rm ./target/release/deps/rocky*
RUN cargo build --release 

# Second Stage
from debian:bookworm-slim

## TODO: needs to statically link openssl to the build, to decrease image size (musl)
## Install shared object needed for rocky (libssl-dev)
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /usr/share/man /usr/share/doc /usr/share/doc-base

# Copy rocky bin and default configuration
COPY --from=build /rocky/target/release/rocky .
COPY ./rocky.toml . 

CMD ["./rocky"]
