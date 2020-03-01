#!/bin/bash

mkdir -p turbo-hearts/assets/{auth,lobby,game}
cp target/x86_64-unknown-linux-musl/release/turbo-hearts turbo-hearts
cp auth.html turbo-hearts/assets/auth/index.html
cp lobby.html turbo-hearts/assets/lobby/index.html
cp packages/game/dist/* turbo-hearts/assets/game
tar czvf turbo-hearts.tgz turbo-hearts/
rm -rf turbo-hearts
