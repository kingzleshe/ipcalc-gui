# IPCalc GUI

[![CI](https://github.com/kingzleshe/ipcalc-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/kingzleshe/ipcalc-gui/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/kingzleshe/ipcalc-gui?label=release)](https://github.com/kingzleshe/ipcalc-gui/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
![Rust 2024](https://img.shields.io/badge/rust-2024-orange.svg)
![Slint 1.16.1](https://img.shields.io/badge/slint-1.16.1-blue.svg)
![Windows](https://img.shields.io/badge/windows-supported-0078d4.svg)

IPCalc GUI is a native desktop IP calculator built with Rust and Slint. It is designed for quick subnet checks, address planning, and day-to-day network troubleshooting without requiring a browser or WebView.

The project is currently Windows-first. The core calculator logic is platform-neutral, but the range list feature opens a temporary text file with Windows Notepad.

## Features

- IPv4 and IPv6 calculator modes.
- IPv4 input as CIDR, address + subnet mask, or address + wildcard mask.
- IPv6 input as standard CIDR notation, including `::` shorthand.
- Network address, broadcast address or last address, subnet mask, wildcard mask, usable host range, address count, and IP type output.
- Light and dark theme support.
- Address range generation, for example `1-20`.
- Native Slint GUI with no WebView dependency.

## Input Examples

| Type | Example | Description |
| --- | --- | --- |
| IPv4 CIDR | `192.168.1.1/22` | Calculates the IPv4 network block |
| IPv4 subnet mask | `192.168.1.1 255.255.252.0` | Detects a contiguous subnet mask |
| IPv4 wildcard mask | `192.168.1.1 0.0.3.255` | Detects a wildcard mask |
| IPv6 CIDR | `2001:db8::1/64` | Calculates the IPv6 network block |
| Address range | `1-20` | Press Enter in the IP range field to generate an address list |

## Requirements

- Rust stable with Rust 2024 edition support.
- Windows 10/11 is recommended.

Install Rust on Windows:

```powershell
winget install Rustlang.Rustup
rustup default stable
```

## Run Locally

```powershell
git clone <repo-url>
cd ipcalc-gui
cargo run
```

## Build

```powershell
cargo build --release
```

The Windows executable is generated at:

```text
target/release/ipcalc.exe
```

## Test

```powershell
cargo test
```

## Release

Release files are built by GitHub Actions when a version tag is pushed. The workflow creates a GitHub Release and uploads a Windows zip file containing `ipcalc.exe`, `README.md`, `LICENSE`, and `CHANGELOG.md`.

To publish a release:

```powershell
git status --short

git add .
git commit -m "Prepare open source release"
git push origin HEAD

git tag -a v0.1.0 -m "IPCalc GUI v0.1.0"
git push origin v0.1.0
```

Pushing normal commits only runs CI. Pushing a `v*` tag, such as `v0.1.0`, triggers the release workflow and creates the GitHub Release.

You can also build the same zip locally:

```powershell
.\scripts\package-windows.ps1
```

The local release package is written to `dist/`.

## Project Layout

```text
.
├── Cargo.toml        # Rust package configuration
├── build.rs          # Slint compilation and Windows resource setup
├── scripts/          # Local packaging helpers
├── src/
│   ├── main.rs       # Application entry point and UI callbacks
│   └── ipcalc.rs     # IPv4/IPv6 calculator logic and tests
└── ui/
    ├── app.slint     # Slint user interface
    └── app-icon.*    # Application icon assets
```

## Contributing

Bug reports, feature requests, and pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

Recommended checks before submitting changes:

```powershell
cargo fmt
cargo test
```

## Security

If you discover a security issue, please read [SECURITY.md](SECURITY.md) and avoid publishing exploitable details in a public issue.

## License

This project is open source under the [MIT License](LICENSE).

## Acknowledgements

This project was inspired by [nicanorflavier/ipnet](https://github.com/nicanorflavier/ipnet), a compact IP subnet calculator CLI. Thanks to that project for the idea and direction.
