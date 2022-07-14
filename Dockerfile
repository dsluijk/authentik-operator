FROM rust:1.62 as build

# Switch workdir.
WORKDIR /akcontroller

# Copy over source files.
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

# Build it all for release.
RUN cargo build --release

# Build from the slim image.
FROM debian:bullseye-slim

# Copy the binary from the base image.
COPY --from=build /akcontroller/target/release/akcontroller .

# Port to serve on.
EXPOSE 8080

# Start the binary.
CMD ["./akcontroller"]
