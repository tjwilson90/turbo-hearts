#!/bin/bash

mkdir -p turbo-hearts/assets/dist/assets
cp target/x86_64-unknown-linux-musl/release/turbo-hearts turbo-hearts
cp -r assets/* turbo-hearts/assets
cp assets/*.jpg turbo-hearts/assets/dist/assets
cp -r assets/cards turbo-hearts/assets/dist/assets
cp packages/game/node_modules/normalize.css/normalize.css turbo-hearts/assets/dist/assets
tar czvf turbo-hearts.tgz turbo-hearts/
rm -rf turbo-hearts
