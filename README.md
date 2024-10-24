# JACKRAMP ZK

This project presents a proof of concept to scale P2P offramp transaction by use of a ZK-powered architecture

### Version
There are 2 version of codes, only SP1 is working at the moment due to error to build `reqwest` lib in `zkRust` version
- SP1
- zkRust 

### Requirements

- [Rust](https://rustup.rs/)
- [Aligned](https://docs.alignedlayer.com/guides/2_build_your_first_aligned_application#app)

## Running the Project

There are 3 main ways to run this project: build a program, execute a program, generate a core proof.

### Build the Program

To build the program, run the following command:

```sh
cd sp1_version/program
cargo prove build
```
This will generate ELF file `sp1_version/program/elf/riscv32im-succinct-zkvm-elf`
### Execute the Program

To run the program without generating a proof:

```sh
cd sp1_version/script
cargo run --release -- --execute
```

This will execute the program and display the output.

### Generate a Core Proof

To generate a core proof for your program:

```sh
cd sp1_version/script
cargo run --release -- --prove
```

### Generate an EVM Proof

```sh
cd sp1_version/script
cargo run --release --bin evm -- --keystore-path <KEYSTORE_PATH> --jackramp-contract-address <JACKRAMP_CONTRACT_ADDR> --rpc-url https://ethereum-holesky-rpc.publicnode.com --network holesky
```