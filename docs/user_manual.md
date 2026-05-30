# User manual

## Installation
You need a functional rust environment to be able to run the project. Easiest way
to install rust is using Rustup. Rustup is a toolchain management application for rust.
Go to this [website](https://rust-lang.org/tools/install/) and use

MacOS: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`  
Win64: Download installer exe, and use it to install Rustup.

All rust tools are installed under home directory of your operating system, such as
`~/cargo/bin` or `C:\Users\[your_user_name]\.cargo`. You need to make sure this is in
your PATH. Rustup installation tries to add this to your PATH, but if it fails you need to add it manually.

Author uses rust version: `rustc 1.87.0` so you might need to install older version
of rust. This can be done with `rustup install 1.87.0`

### Verification
After installation running `rustc --version` on your console should return your rust version.

## Running 
Building the project from workspace root can be done with `cargo build` Workspace root
is the project root when cloned from git. Running build will crate a target binary with debug
settings under target/debug. Currently there are two binaries: tui which is the Text Based UI to run 
the game and benchmark which is a commandline tool to simulate AI playing multiple times in row
generating a run report. tui and benchmark are executables you can run from command line.

[To be completed more later...]


### Other
To be completed later...