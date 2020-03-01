#!/bin/bash

mkdir -p turbo-hearts/assets/{auth,lobby,game}
cp target/x86_64-unknown-linux-musl/release/turbo-hearts turbo-hearts
cp -r assets turbo-hearts
tar czvf turbo-hearts.tgz turbo-hearts/
rm -rf turbo-hearts
