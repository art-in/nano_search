FROM rust:latest

# install OS packages
RUN apt update
RUN apt install -y \
  # used by gungraun (bench harness)
  valgrind

# install and use rust nightly toolchain by default. reasons:
# - to enable some useful unstable rustfmt options (see rustfmt.toml)
# - to enable legacy mangling format in profiling build (see Cargo.toml)
RUN rustup toolchain install nightly
RUN rustup override set nightly

# install rust components
RUN rustup component add --toolchain nightly rustfmt clippy rust-analyzer

# install tools
RUN cargo install --locked samply # CPU profiler
RUN cargo install --version 0.19.0 gungraun-runner # bench runner

