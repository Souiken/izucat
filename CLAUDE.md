# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

izucat is a Rust CLI tool that recursively concatenates text and binary files into Typst format. It can process directories, single files, command output, or stdin, generating formatted Typst documents with syntax highlighting and line numbers.

## Build and Development Commands

```bash
# Build release version
cargo build --release

# Build debug version
cargo build

# Run directly
cargo run -- [OPTIONS] -o <FILE> [input]

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Architecture

### Single-file Structure
The entire application is in `src/main.rs` with these key components:

- `is_binary()`: Detects binary files by checking for null bytes and non-text characters
- `escape_typst()`: Escapes text for Typst format (currently a pass-through)
- `to_hex_view()`: Converts binary data to hex dump format with ASCII preview
- `generate_typst()`: Main logic that handles three input modes:
  1. stdin (piped input)
  2. Command execution (`--cmd`)
  3. File/directory processing
- `main()`: CLI argument parsing using clap

### Output Format
Generated Typst files include:
- Custom page setup (A4, 2cm margins)
- Unifont monospace font at 8pt
- Custom `codeblock()` function for line numbering
- Content wrapped in quadruple backticks (`\u{0060}` used to avoid Typst parsing issues)

### File Processing Logic
- Uses `ignore` crate's WalkBuilder for directory traversal (respects .gitignore)
- Binary files: displayed as hex dumps
- Text files: displayed with optional line numbers
- Non-UTF8 files: treated as binary
- Progress bar shown for multi-file operations

## Key Implementation Details

- Backticks in Typst output use Unicode escape `\u{0060}` to avoid conflicts with Typst's raw block syntax
- Line numbers are implemented via custom Typst function with offset logic (starts at line 3, displays as line 1)
- Single file input doesn't display full path, only filename
- Directory processing adds pagebreaks between files
