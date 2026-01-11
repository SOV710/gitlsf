# gitlsf

![CI](https://img.shields.io/github/actions/workflow/status/SOV710/gitlsf/ci.yml?style=flat-square&logo=github&label=CI)

A fast Git repository line counter written in Rust.

## Features

- **Fast**: Uses parallel processing with rayon for efficient counting
- **Smart filtering**: Automatically excludes binary, configuration, and documentation files
- **Multiple output modes**: Verbose, quiet, and summary modes
- **Git-aware**: Only counts files tracked by Git

## Installation

### From crates.io

```bash
cargo install gitlsf
```

### From source

```bash
git clone https://github.com/SOV710/gitlsf.git
cd gitlsf
cargo install --path .
```

### Pre-built binaries

Download pre-built binaries from the [Releases](https://github.com/SOV710/gitlsf/releases) page.

## Usage

```bash
# Count lines in current directory
gitlsf

# Count lines in a specific directory
gitlsf /path/to/repo

# Quiet mode - only show total
gitlsf -q

# Summary mode - show file count and total lines
gitlsf -s

# Verbose mode (default) - show each file
gitlsf -v
```

### Command-line options

```
Usage: gitlsf [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to the Git repository (defaults to current directory)

Options:
  -v, --verbose  Verbose mode - show each file with its line count (default)
  -q, --quiet    Quiet mode - only show the total line count
  -s, --summary  Summary mode - show total lines and file count
  -h, --help     Print help
  -V, --version  Print version
```

### Output examples

**Default (verbose) mode:**
```
   3 src/main.rs
   2 src/lib.rs
   5 total
```

**Quiet mode (`-q`):**
```
5
```

**Summary mode (`-s`):**
```
Files: 2
Lines: 5
```

## Filtered file types

gitlsf automatically excludes the following file types:

**Media files:** `.mp3`, `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg`, `.woff2`, `.ico`, `.webp`, `.bmp`, `.tiff`, `.wav`, `.mp4`, `.avi`, `.mov`, `.webm`, `.flac`, `.ogg`, `.ttf`, `.woff`, `.eot`, `.otf`, `.pdf`

**Data files:** `.mmdb`, `.csv`, `.json`, `.toml`, `.lock`, `.ini`, `.yaml`, `.yml`, `.xml`

**Documentation:** `.md`

**Special files:** `LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`, `.gitignore`

## Performance

gitlsf is designed to be fast even on large repositories:

- Uses `git ls-files` for efficient file listing
- Parallel line counting with rayon
- Memory-efficient streaming file reads

## Development

### Building

```bash
cargo build
```

### Running tests

```bash
cargo test
```

### Running lints

```bash
cargo clippy -- -D warnings
```

### Formatting

```bash
cargo fmt
```

### Building documentation

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
