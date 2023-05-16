#!/bin/sh

./build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying contract"

# https://docs.near.org/tools/near-cli#near-dev-deploy
near deploy crossword.psychehose.testnet --wasmFile ./target/wasm32-unknown-unknown/release/crossword_puzzle.wasm

near deploy crossword.psychehose.testnet --wasmFile ./target/wasm32-unknown-unknown/release/crossword_puzzle.wasm \
  --initFunction 'new' \
  --initArgs '{"solution": "69c2feb084439956193f4c21936025f14a5a5a78979d67ae34762e18a7206a0f"}'
