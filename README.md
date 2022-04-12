# Static alias analyzer for Unsafe Rust

The project idea is to analyze the MIR code of a Rust program and detect
posible cases where the Safe Rust guarantees of aliasing are broken after
the usage of unsafe operations.

# Important notes
* This project only works on *linux* for now

## Basic Usage

1. Install Rust from <https://www.rust-lang.org/>
1. Execute `cargo build` to install the project dependencies and generate the executable
1. Use command `cargo run $filename` to run the project

## Advanced Usage
Taken into account the default rust installation.
1. Add this enviromental variables to your system (in *.bashrc* or *.zshrc*)
```
export RUST_CHANNEL=nightly-2022-01-01

export RUSTFLAGS="-L $HOME/.rustup/toolchains/${RUST_CHANNEL}-x86_64-unknown-linux-gnu/lib"
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:$HOME/.rustup/toolchains/${RUST_CHANNEL}-x86_64-unknown-linux-gnu/lib"
```
1. Execute `cargo install --path .` to install the project dependencies and add the `cargo rsaa` command
1. Go to another cargo project and run `cargo rsaa` to run the analysis on the crate. (*needs a **main** file/function*)