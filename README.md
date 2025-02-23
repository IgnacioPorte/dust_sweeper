# 🧹 dust_cleaner

[![Crates.io](https://img.shields.io/crates/v/dust_sweeper)](https://crates.io/crates/dust_sweeper)
[![Docs.rs](https://docs.rs/dust_cleaner/badge.svg)](https://docs.rs/dust_sweeper)
[![License: MIT](https://img.shields.io/crates/l/dust_sweeper)](LICENSE)

**A Rust-based tool to identify and remove dust UTXOs from a Bitcoin wallet in a privacy-preserving way.**  
Supports sweeping dust to a **burn address** or **consolidating dust safely**.

---

## 📦 Installation

### **From Crates.io**
Install the CLI using `cargo`:

```bash
cargo install dust_cleaner
```

## Usage

```bash
dust_sweeper_cli [OPTIONS] --rpc <RPC_URL> --user <USERNAME> --pass <PASSWORD>

Options:
    -r, --rpc <RPC_URL>         Bitcoin Core RPC URL (e.g., http://localhost:18443)
    -u, --user <USERNAME>       RPC username
    -p, --pass <PASSWORD>       RPC password
    -t, --threshold <AMOUNT>    Dust threshold in sats [default: 1000]
        --dry-run              Dry-run mode (only list dust UTXOs, no PSBT created)
        --burn-address <ADDR>  Burn address to send dust to [default: 1BitcoinEaterAddressDontSendf59kuE]
        --fee <AMOUNT>         Fixed fee in sats [default: 500]
    -h, --help                 Print help information
```

Example output:
```
🔍 Found 2 dust UTXOs:
💰 UTXO: 800 sats
💰 UTXO: 600 sats
Using fixed fee: 500 sats

🔥 PSBT #1 (Burning dust to 1BitcoinEaterAddressDontSendf59kuE):
cHNidP8BAH4CAAAAAr4S/3p+... [base64 PSBT data]
```

## 🔧 Features

- Identify dust UTXOs below a configurable threshold
- Privacy-preserving dust removal by:
  - Grouping UTXOs by address
  - Creating separate transactions for each address
- Support for burning dust to a specified address

## 🛠️ Building from Source

1. Clone the repository:
```bash
git clone https://github.com/IgnacioPorte/dust_sweeper
cd dust_sweeper
```

2. Build the project:
```bash
cargo build --release
```

3. The binary will be available at `target/release/dust_sweeper_cli`

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## ⚠️ Disclaimer

This tool is provided as-is. Always verify transactions before signing and broadcasting. Test thoroughly on regtest/testnet before using on mainnet.