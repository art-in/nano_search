FROM rust:latest

# install rust nightly toolchain. reasons:
# - to enable some useful unstable rustfmt options (see rustfmt.toml)
# - to enable lagacy mangling format in profiling build (see Cargo.toml)
RUN rustup toolchain install nightly
RUN rustup override set nightly

# install rust components
RUN rustup component add --toolchain nightly rustfmt clippy rust-analyzer

# install tools
RUN cargo install --locked samply
