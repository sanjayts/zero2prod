# Background

This repository holds the code written as part of reading the "Zero to production" Rust book.

# Changes

This book has a few changes when compared to the official code provided in the book. These are:

1. Using a newer version of sqlx-cli locally
2. Using a newer version of sqlx-cli in our GitHub pipeline -- the official code uses an older version 0.5 which didn't require any additional TLS related features to be provided during installation
3. The official version uses docker which is a royal PITA to set up on Mac M1 given the lack of a proper CLI version (I really didn't want to use Docker desktop). This version uses podman which works pretty well on both Linux and Mac.
4. Additional documentation where required to specify *why* something was done the way it was done.