# The latest stable version as of 29th Aug 2022
FROM rust:1.63.0

WORKDIR /app

# Update system deps required for building code
RUN apt update && apt install lld clang -y

# Copy the contents of our workspace to /app folder
COPY . .

RUN cargo build --release

ENTRYPOINT ["./target/release/zero2prod"]