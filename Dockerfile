# Use the Rust image from Docker Hub
FROM rust:latest as build

# Create a new directory for our project
WORKDIR /usr/src/my-http-server

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Build the dependencies to cache them
RUN cargo build --release

# Copy the rest of the source code to the container
COPY src ./src

# Build the server in release mode
RUN cargo build --release

# Create a new image with only the necessary files
FROM debian:buster-slim
COPY --from=build /usr/src/my-http-server/target/release/my-http-server /usr/local/bin/my-http-server

# Expose the port on which the server will listen
EXPOSE 8080

# Start the server
CMD ["my-http-server"]