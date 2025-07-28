#!/bin/bash
set -e
source $HOME/.cargo/env
cargo install rust-script # Install rust-script (for json DFA -> serialized DFA script)
exec "$@"
