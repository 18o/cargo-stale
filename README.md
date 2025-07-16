# cargo-stale

A fast, concurrent tool to check for outdated dependencies in your Rust `Cargo.toml` file.

## Features

- Fast concurrent dependency checking using async/await
- Smart semantic version comparison and compatibility ranges
- Support for normal, dev, build, and workspace dependencies
- Flexible filtering to show outdated or all dependencies
- Clean table output with current vs latest versions
- Multiple command-line options for different use cases
- Works as both cargo subcommand and standalone tool
- Workspace member support with dependency source tracking

## Installation

Install from crates.io

```bash
cargo install cargo-stale
```

Or install from Git:

```bash
cargo install --git https://github.com/18o/cargo-stale
```

Or clone and build locally:

```bash
git clone https://github.com/18o/cargo-stale
cd cargo-stale
cargo build --release
```

## Usage

### As a Cargo Subcommand (Recommended)

Once installed, you can use it as a cargo subcommand:

```bash
# Basic usage
cargo stale

# Check only outdated dependencies
cargo stale --outdated-only

# Include build dependencies with verbose output
cargo stale --build-deps --verbose

# Check a specific Cargo.toml file
cargo stale --manifest /path/to/Cargo.toml
```

### As a Standalone Tool

You can also run it directly:

```bash
# Basic usage
cargo-stale

# With options
cargo-stale --outdated-only --verbose
```

### Command Line Options

```bash
cargo stale [OPTIONS]

Options:
  -m, --manifest <MANIFEST>    Path to Cargo.toml file [default: Cargo.toml]
  -o, --outdated-only         Show only outdated dependencies
  -b, --build-deps            Include build dependencies
  -v, --verbose               Verbose output
  -h, --help                  Print help
  -V, --version               Print version
```

## Sample Output

```
🚀 Starting cargo-stale...

📊 Dependency Check Results:
--------------------------------------------------------------------------------------------------------------
Dependency                          Current Version      Latest Version       Status           Source
--------------------------------------------------------------------------------------------------------------
anyhow (workspace)                  1                    1.0.98               ✅ Latest         root
async-trait (workspace)             0.1                  0.1.88               ✅ Latest         root
bincode (workspace)                 2                    2.0.1                ✅ Latest         root
chrono (workspace)                  0.4                  0.4.41               ✅ Latest         root
dashmap (workspace)                 6.1                  7.0.0-rc2            🟡 Outdated (Pre) root
futures-util (workspace)            0.3                  0.3.31               ✅ Latest         root
log (workspace)                     0.4                  0.4.27               ✅ Latest         root
rayon (workspace)                   1.10                 1.10.0               ✅ Latest         root
rustls (workspace)                  0.23                 0.23.29              ✅ Latest         root
serde (workspace)                   1                    1.0.219              ✅ Latest         root
serde_json (workspace)              1                    1.0.140              ✅ Latest         root
tokio (workspace)                   1                    1.46.1               ✅ Latest         root
tokio-tungstenite (workspace)       0.27                 0.27.0               ✅ Latest         root
tracing (workspace)                 0.1                  0.1.41               ✅ Latest         root
tracing-subscriber (workspace)      0.3                  0.3.19               ✅ Latest         root
uuid (workspace)                    1.16                 1.17.0               🔴 Outdated       root
anyhow                              1                    1.0.98               ✅ Latest         client
futures-util                        0.3                  0.3.31               ✅ Latest         client
http                                1.3                  1.3.1                ✅ Latest         client
rand                                0.9                  0.9.1                ✅ Latest         client
rustls-pki-types                    1.12                 1.12.0               ✅ Latest         client
serde_json                          1                    1.0.140              ✅ Latest         client
tauri                               2                    2.6.2                ✅ Latest         client
tauri-plugin-clipboard-manager      2                    2.3.0                ✅ Latest         client
tauri-plugin-geolocation            2                    2.3.0                ✅ Latest         client
tauri-plugin-http                   2                    2.5.0                ✅ Latest         client
tauri-plugin-opener                 2                    2.4.0                ✅ Latest         client
tauri-plugin-os                     2                    2.3.0                ✅ Latest         client
tauri-plugin-store                  2                    2.3.0                ✅ Latest         client
thiserror                           2.0.12               2.0.12               ✅ Latest         client
serde                               1                    1.0.219              ✅ Latest         shared
--------------------------------------------------------------------------------------------------------------
```

## How It Works

cargo-stale reads your `Cargo.toml` file and concurrently queries the crates.io API to check for the latest version of each dependency. It uses intelligent semantic version comparison to determine if a dependency is actually outdated based on your version requirements:

### Version Compatibility Rules

- **`"1"` or `"^1.0"`** - Compatible with 1.x.x series, only outdated when 2.x.x is available
- **`"^0.10"`** - Compatible with 0.10.x series, outdated when 0.11.x is available (0.x versions are more restrictive)
- **`"~1.2"`** - Compatible with 1.2.x series, outdated when 1.3.x or higher is available
- **`"=1.2.3"`** - Exact version, outdated when any newer version is available
- **`">=1.0"`, `">1.0"`, etc.** - Range requirements are not considered outdated

This follows [Semantic Versioning](https://semver.org/) and [Cargo's version requirement specifications](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html).

## TODO

- [ ] Automatically update Cargo.toml with latest versions (add `--update` flag)
- [ ] Interactive mode for selective dependency updates
- [ ] Support for private registries and alternative registries
- [ ] Configuration file support for custom rules

## System Requirements

- Rust 1.70.0 or later
- Internet connection (to query crates.io API)

## Dependencies

- `tokio` - Async runtime for concurrent requests
- `reqwest` - HTTP client for crates.io API
- `clap` - Command line argument parsing
- `serde` - JSON deserialization
- `toml` - TOML file parsing
- `anyhow` - Error handling
- `env_logger` & `log` - Logging support

## Performance

cargo-stale is designed for speed:

- **Concurrent requests**: All dependencies are checked simultaneously
- **Minimal dependencies**: Uses only essential crates
- **Smart caching**: HTTP client connection reuse
- **Efficient parsing**: Fast TOML and JSON processing

Typical performance: Checking 10 dependencies takes ~1-2 seconds (depending on network).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Similar Tools

- **`cargo-outdated`** - More comprehensive but slower, shows detailed dependency trees
- **`cargo-audit`** - Focuses on security vulnerabilities rather than version updates
- **`cargo-edit`** - Helps manage dependencies but doesn't check for updates
- **`cargo-update`** - Updates installed cargo binaries, not project dependencies

## Why cargo-stale?

cargo-stale strikes the perfect balance between functionality and performance:

✅ **Fast**: Concurrent checking makes it much faster than sequential tools  
✅ **Smart**: Understands semantic versioning and compatibility ranges  
✅ **Simple**: Clean, easy-to-read output without overwhelming details  
✅ **Reliable**: Respects your version requirements and doesn't suggest breaking changes  
✅ **Convenient**: Works as both a cargo subcommand and standalone tool

Perfect for quick dependency checks in your daily development
