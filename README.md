# Rust ZIP Cracker v3.5

A fast, accurate, and feature-rich brute-force password cracking tool for ZIP archives, written in Rust. This tool is designed to maximize multi-core CPU utilization, ensure memory safety, and keep resource usage minimal.


## Description

This project aims to provide a reliable security auditing tool to test the strength of passwords on ZIP archives. It is perfect for research and educational purposes within a controlled lab environment. The design combines high performance of an efficient engine, memory safety and concurrency of Rust, and the convenience of a mature command-line application.

This release has specifically fixed all logical bugs, including false positives and handling of non-standard ZIP file types, to ensure absolute accuracy.

## Key Features

* 🚀 **Concurrent & Fast:** Uses Rust native threads to run password attempts in parallel, leveraging all available CPU cores.
* 🧠 **Memory-Efficient:** Streams the wordlist line by line using `BufReader`, allowing it to handle wordlists dozens of gigabytes in size without consuming excessive RAM.
* ✔️ **Reliable & Accurate Logic:** No longer relies on unreliable metadata. The cracking logic fundamentally tests each password against every file inside the archive to guarantee no false positives.
* ✨ **Professional Interface:** Built with `clap` for advanced command-line argument parsing and `indicatif` for a real-time, informative progress bar.
* 🛑 **Graceful Shutdown:** Uses `ctrlc` to capture Ctrl+C signals, ensuring the program can stop cleanly, save progress, and display a final report without corrupting the terminal output.
* 📂 **Automatic Extraction:** If a password is found, the tool can automatically extract the ZIP contents to a directory specified with the `-o` flag.

## Installation

### Prerequisites

* Rust and Cargo: [Install Rust](https://www.rust-lang.org/tools/install)
* Git

You can install and run this tool in two ways:

### 1. From Source (Recommended)

This method is best if you want to modify or manually build the project.

```bash
# 1. Clone the repository (replace URL with your repo URL)
git clone https://github.com/mfachri88/RustZipCracker.git

# 2. Change into the project directory
cd RustZipCracker

# 3. Build the optimized release executable
cargo build --release
```

After building, the high-performance executable will be available at `target/release/rust_zip_cracker`.

### 2. Via `cargo install`

This method installs the binary directly from the Git repository and makes it available as a terminal command.

```bash
cargo install --git https://github.com/mfachri88/RustZipCracker.git
```

## Usage

Run the tool from the project’s root directory:

```bash
# Run the built executable
target/release/rust_zip_cracker -f <file.zip> -w <wordlist.txt> [other_options]

# Or run without separate build
tools/run.sh -f <file.zip> -w <wordlist.txt> [other_options]
```

> **Note:** Use `--` after `--release` when running with Cargo to ensure arguments are passed to the tool, not to Cargo itself.

## Command-Line Flags

| Flag | Type     | Description                                            | Required |
| ---- | -------- | ------------------------------------------------------ | -------- |
| `-f` | `string` | Path to the ZIP file to crack.                         | Yes      |
| `-w` | `string` | Path to the wordlist file containing passwords.        | Yes      |
| `-o` | `string` | Output directory for extracted files.                  | No       |
| `-t` | `int`    | Number of threads/workers to use (default: CPU cores). | No       |

## Examples

### 1. Basic Usage

Attempt to find the password with default settings:

```bash
./target/release/rust_zip_cracker -f secret-archive.zip -w /usr/share/wordlists/rockyou.txt
```

### 2. Using 16 Threads

Maximize CPU utilization by setting 16 workers:

```bash
./target/release/rust_zip_cracker -f project.zip -w passwords.txt -t 16
```

*Output:*

```
[===>     ] 10% | 1,434,439/14,344,393 | 15,023 pwd/s
```

### 3. Crack & Extract in One Step

Crack and automatically extract contents to `extracted_output` directory:

```bash
./target/release/rust_zip_cracker -f important.zip -w rockyou.txt -o extracted_output
```

## Ethical Warning & License

> **Warning:** This tool is intended for educational and research purposes in lawful, controlled environments. Using it to attack targets you do not own or without explicit permission is illegal. The developer is not responsible for misuse.

This project is licensed under the MIT License.
