#!/bin/sh

# build the rust project
cargo build --release

# create a symlink in ~/.local/bin (common user bin dir)
mkdir -p "$HOME/.local/bin"
ln -sf "$PWD/target/release/waifu" "$HOME/.local/bin/waifu"

echo "waifu is now available as a command. make sure $HOME/.local/bin is in your PATH."
