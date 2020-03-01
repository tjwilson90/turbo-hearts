#!/bin/bash

mkdir -p turbo-hearts/assets/{auth,lobby,game}
cp target/x86_64-unknown-linux-musl/release/turbo-hearts turbo-hearts
cp packages/game/dist/* turbo-hearts/assets/game
cp -r assets turbo-hearts
tar czvf turbo-hearts.tgz turbo-hearts/
rm -rf turbo-hearts
