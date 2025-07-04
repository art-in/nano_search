FROM rust:latest

# install rust nightly toolchain:
# - few useful unstable rustfmt options work in nightly only (see rustfmt.toml)
RUN rustup toolchain install nightly
RUN rustup override set nightly

# install rust components
RUN rustup component add --toolchain nightly rustfmt clippy

# install tools
RUN cargo install --locked samply
