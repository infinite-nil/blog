FROM rust:latest as builder

# Make a fake Rust blog to keep a cached layer of compiled crates
RUN USER=root cargo new blog
WORKDIR /usr/src/blog
COPY Cargo.toml Cargo.lock ./
# Needs at least a main.rs file with a main function
RUN mkdir src && echo "fn main(){}" > src/main.rs
# Will build all dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/src/blog/target \
  cargo build --release

# Copy the rest
COPY . .
# Build (install) the actual binaries
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/src/blog/target \
  cargo install --path .

# Runtime image
FROM debian:bullseye-slim

# Run as "blog" user
RUN useradd -ms /bin/bash blog

USER blog
WORKDIR /blog

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/local/cargo/bin/blog /blog/blog

# No CMD or ENTRYPOINT, see fly.toml with `cmd` override.