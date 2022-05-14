FROM rust:1.60.0-slim-buster as build

# Install prerequisites
RUN apt-get update -y && apt-get install -y pkg-config libssl-dev

# Create a new empty shell project
RUN cargo new --bin nomadcoin-rs
WORKDIR /nomadcoin-rs

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Cache dependencies
RUN cargo build --release --locked && rm src/*.rs

# Copy source tree
COPY ./src ./src

# # Build for release
RUN cargo build --release --locked
RUN rm ./target/release/deps/nomadcoin_rs*

# Final base image
FROM rust:1.60.0-slim-buster

# Copy the build artifact from the build stage
COPY --from=build /nomadcoin-rs/target/release/nomadcoin-rs .
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

# Startup command
ENTRYPOINT ["./nomadcoin-rs"]