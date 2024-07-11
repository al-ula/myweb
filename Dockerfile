# # Use the official Rust image as the base image
# FROM rust-musl-cross:x86_64-musl as builder

# # Set the working directory in the container
# WORKDIR /usr/src/myweb

# # Copy the entire project
# COPY . .

# # Build the application
# RUN cargo build --release --target x86_64-unknown-linux-musl

# Start a new stage with a minimal image
FROM alpine

# Set the working directory in the container
WORKDIR /usr/local/bin

# Copy the binary from the builder stage
# COPY --from=builder /usr/src/myweb/target/release/myweb .

COPY target/x86_64-unknown-linux-musl/release/myweb /usr/local/bin

# Copy the necessary directories
COPY articles /usr/local/bin/articles
COPY theme/default/static /usr/local/bin/theme/default/static
COPY theme/default/templates /usr/local/bin/theme/default/templates
COPY config /usr/local/bin/config

# Expose the port your application listens on (change if necessary)
EXPOSE 8000

# Run the binary
CMD ["./myweb"]
