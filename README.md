# Static alias analyzer for Unsafe Rust

The project idea is to analyze the MIR code of a Rust program and detect
posible cases where the Safe Rust guarantees of aliasing are broken after
the usage of unsafe operations.

## Usage

1. Install Rust from <https://www.rust-lang.org/>
1. Execute `cargo build` to install the project dependencies and generate
the executable
1. Use command `cargo run` to run the project
