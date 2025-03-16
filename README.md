# Ethereum Vanity Address Generator

A high-performance Rust program that generates Ethereum vanity addresses with custom prefixes and suffixes.

## Features

- Generate Ethereum addresses with custom prefixes and/or suffixes
- Multi-threaded for maximum performance
- Real-time performance metrics (keys/second)
- Case-insensitive matching
- Uses industry-standard cryptographic libraries
- Written in safe Rust

## Building

Make sure you have Rust installed. Then:

```bash
cargo build --release
```

The executable will be in `target/release/eth-key-gen`

## Running

Basic usage:
```bash
cargo run --release -- --prefix dead --suffix beef
```

Available options:
- `-p, --prefix <PREFIX>`: Desired address prefix (without 0x)
- `-s, --suffix <SUFFIX>`: Desired address suffix
- `-t, --threads <THREADS>`: Number of threads to use (default: number of CPU cores)
- `-q, --quantity <QUANTITY>`: The number of addresses to generate

Examples:
```bash
# Find address starting with "cafe"
cargo run --release -- --prefix cafe

# Find 3 addresses starting with "cafe"
cargo run --release -- --prefix cafe --quantity 3

# Find address ending with "dead"
cargo run --release -- --suffix dead

# Find address starting with "abc" and ending with "000"
cargo run --release -- --prefix abc --suffix 000

# Use 8 threads
cargo run --release -- --prefix dead --threads 8
```

## Performance

The program uses:
- Multi-threading via rayon
- Efficient cryptographic operations
- Lock-free counters for performance metrics
- Real-time speed monitoring

Performance varies by:
- CPU speed and number of cores
- Desired prefix/suffix length (longer = slower)
- System load

## Security Notes

- Private keys are generated using your operating system's secure random number generator
- Never share your private keys with anyone
- Store generated private keys securely
- This is for educational purposes - use at your own risk

## Dependencies

- secp256k1: For elliptic curve operations
- rand: For secure random number generation
- sha3: For Keccak256 hashing
- ethereum-types: For Ethereum address formatting
- hex: For hexadecimal encoding
- rayon: For parallel processing
- clap: For command-line argument parsing
- indicatif: For progress indicators 

## Benchmarked (Ryzen 8945HS 8 Cores 16 Threads)

Hash Rate: ~400000 or ~400k keys/s
| Prefix                    | Time (seconds)                                            |
| --------------------------| --------------------------------------------------------- |
| dead                      | 0.02 |
| 00000                     | 5.35 |
| abcd                      | 0.34 |
| 000000                    | 40   |

## Contributing
Feel free to add your own benchmarked values.