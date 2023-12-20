#!/bin/bash

# dive into localhost folder
cd localhost

# build release executable
cargo build --release

# jump up to root folder
cd ..

# copy release executable to the root folder level with nice name
cp ./localhost/target/release/localhost ./runme