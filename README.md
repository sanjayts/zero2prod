# Background

This repository holds the code written as part of reading the "Zero to production" Rust book.

# Changes

This book has a few changes when compared to the official code provided in the book. These are:

1. Using a newer version of sqlx-cli locally
2. Using a newer version of sqlx-cli in our GitHub pipeline -- the official code uses an older version 0.5 which didn't require any additional TLS related features to be provided during installation
3. The official version uses docker which is a royal PITA to set up on Mac M1 given the lack of a proper CLI version (I really didn't want to use Docker desktop). This version uses podman which works pretty well on both Linux and Mac.
4. Additional documentation where required to specify *why* something was done the way it was done.

# Testing

How do we go about testing this application? The simplest would be to run the `cargo test` command which will run both our integration and unit tests. My usual testing session looks something like the below:

```shell
# Start the docker container locally and perform migration
REMOVE_CONTAINER=true ./scripts/init_db.sh

# Run test without stdout enabled
cargo test

# Run test with stdout enabled for debugging tests
TEST_LOG=1 cargo test | bunyan

# Start up the server with trace log level enabled
RUST_LOG=trace cargo run

# Run a couple of successful and failure curl commands
curl -X POST http://localhost:8080/subscriptions -H "Content-Type: application/x-www-form-urlencoded" -d "name=Sanjay&email=sanjay@hotmail.com" # works

curl -X POST http://localhost:8080/subscriptions -H "Content-Type: application/x-www-form-urlencoded" -d "name=Sanjay&email=sanjay@hotmail.com" # fails; same email

curl -X POST http://localhost:8080/subscriptions -H "Content-Type: application/x-www-form-urlencoded" -d "name=Sanjay&email=sa@hotmail.com" # Works, same name but different emails so fine!
```