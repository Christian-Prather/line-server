#!/bin/bash

# Make sure binary is up to date
./build.sh

# Run server
echo "Running executable"
cargo run --release -- $1