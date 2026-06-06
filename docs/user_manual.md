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

### Code coverage
The project uses llvm-cov as test coverage tool. This isn't part of default cargo installation so if you want to run 
the test coverage locally you need to install the llvm-cov extension. This can be done with command:

`cargo install cargo-llvm-cov`

Please, note that llvm-cov requires at least `rustc 1.87.0`.

## Building 
Building the project from workspace root can be done with `cargo build`   

Workspace root is the project root when cloned from git. Running build will crate a target binary with debug
settings under target/debug. Currently, there are two binaries: tui which is the Text Based UI to run 
the game and benchmark which is a commandline tool to simulate AI playing multiple times in row
generating a run report. Tui and benchmark are executables you can run from command line.

Release version can be built with `cargo build --release`.  

This will create a target binary without any debug information under target/release. The executable can be run (depending
on your OS) by executing the output file under release folder. NOTE: make sure you have execute / run permissions
for the binary.

### Crate documentation
Crate documentation can be built with `cargo doc --no-deps --open`. Alternatively this can be run with bash script `generate_crate_documentation.sh` This will crate documentation from different crates
in the workspace. It doesn't include the dependencies and tries to open it with default associated program, usually a browser.
Note: The documentation is only focused on the AI and engine crates. Tui and Benchmark, not being priorities are not
documented currently.

## Running 
### Unit tests
Unit tests can be run with `cargo test`. It will output the test results to the console.
### Code coverage
Code coverage can be run with a script `run_code_coverage_html.sh` in project root. Or if you don't have bash compatible
interpreter then with command   
`cargo llvm-cov --html --open`  

This will run the tests, create a html output and tries to open it with default program associated to the type. (usually a browser of some sort)

### Text based UI (TUI)
#### Description
Tui is a very simple text based UI for the ***Connect Four*** game. It allows two types of players (Human or AI) to play against each other. If one of the players
is Human then input from user is expected. Input is given with the keyboard using column numbers from 0 to 6. If one of the 
players is AI, then some additional settings are needed. If 2 AIs are selected, they will autoplay against each other.

The extra settings for AI: 
* ***Heuristic version.*** Options: v1 and v2. v1 means, there's no heuristic function used, so game state value on the max depth isn't being valuated by the heuristic function.
v2 means a simple heuristic evaluation at max depth will be done. This should (in theory) try to guide the AI selecting a board state  more favourable to it.
* ***AI Mode.*** Options: Fixed Depth or Time Limited. Fixed Depth means the search is limited to a fixed depth. It will try to find a best solutions but doesn't go beyond this given depth. 
Time Limited means, AI turn is limited by time in milliseconds. This method uses iterative deepening and does one depth at the time until the  time budget is exhausted and returns 
the best result found so far. Please note: The time constraint resolution isn't perfect as it estimates time remaining, and can potentially overshoot.
* ***Time limit.*** TIme limit in milliseconds if you chose Time Limited search. Good values to start with would be 50ms.
* ***Depth limit.*** If Fixed depth is used, the max depth search will go to. Depending on your hardware, starting values under 10 are good.

You need to build the binary first and go to `target/[debug/release]` folder and run the `tui` executable. On Win64 it's `tui.exe` and on MacOS
it's `tui`

You can also build and run it by specifying the project (and type) with cargo with command `cargo run --bin tui`

### Benchmark tool
#### Description
Benchmark tool is a command line tool to run two versions of AI against each other. The benchmark will collect some data form all the runs
and create a json formatted report of the run. It supports various command line arguments to create different type of runs.

Options:
* --help (Outputs the help)
* --games (Number of games run, default 50)
* --time-ms (Optional argument if time limited search is used, enables iterative deepening!)
* --output (Output file path, default results.json)
* --alpha-beta (If alpha beta pruning should be used, default: true)
* --heuristic-min (Heuristic version of Min (the minimizer), default: v1)
* --heuristic-max (Heuristic version of Max (the maximizer), default: v1)

Under project root there are some bash scripts like `run_benchmark_v2.sh` which run the tool for 50 games having
both AI players using heuristic v2 with max depth of 6. Inspection of the parameters give you good indication how to use the tool.

### Other
#### Other useful command and tips:
`rustup toolchain list` Lists your current toolchains if you have already existing Rust installations.  
`rustup default [your toolchain name from list]` If you want to switch your active (default) toolchain version.  
`cargo clean` when you want to clean the workspace.


### TLDR version
1. Install rust (rustup)
2. Create documentation `generate_crate_documentation.sh` or `cargo doc --no-deps --open`. Read the docs.
3. Build it with `cargo build --release`
4. Under `target/release` folder , either run tui or benchmark binaries.
