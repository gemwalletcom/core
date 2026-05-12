# Gem Wallet Core

[![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![GitHub release](https://img.shields.io/github/v/release/gemwalletcom/core)](https://github.com/gemwalletcom/core/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/gemwalletcom/core)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=flat&logo=telegram&logoColor=white)](https://t.me/gemwallet_developers)
![GitHub Repo stars](https://img.shields.io/github/stars/gemwalletcom/core?style=social)

[![Unit Tests](https://github.com/gemwalletcom/core/actions/workflows/ci.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/ci.yml)
[![Docker Build](https://github.com/gemwalletcom/core/actions/workflows/docker.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/docker.yml)
[![iOS Tests](https://github.com/gemwalletcom/core/actions/workflows/ci-stone-ios.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/ci-stone-ios.yml)
[![Android Build](https://github.com/gemwalletcom/core/actions/workflows/ci-stone-android.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/ci-stone-android.yml)

# Introduction

Gem Wallet Core is the core engine powering [Gem Wallet](https://gemwallet.com/), a fully open source, secure and decentralized crypto wallet designed for Bitcoin, Ethereum, Solana, BNB Chain, Base, Sui and much more. Built in Rust, it ensures high performance, safety, and reliability.

## Gem Wallet Features:

- 🚀 High-Performance: Completely native UI and Core is written in Rust for speed and safety.
- 🔐 Secure: Utilizes strong cryptographic standards.
- 🛠 Extensible: Designed to support additional features and integrations.
- 🤝 Open Source: Community-driven and actively maintained.

Gem Wallet Core serves as the backbone for both backend and frontend apps, handling various tasks, including:

- Transaction indexing and push notifications
- Asset price, charts and alerts
- Fiat on and off-ramps
- ENS, Solana and more name resolution
- NFTs
- Native and cross-chain swaps
- Native BNB Chain and Sui staking
- Hyperliquid perpetual futures trading
- More
- ...

## Running API

### Install dependencies

Run `just install` to install rust, typeshare

### Setup DB

- Create a new database `api` and grant privileges to `username` role
- Run `diesel migration run` to create tables and do migrations

Run API locally: `cargo run --package api`

## Security Scanning

Run `just audit` to execute [`cargo-audit`](https://github.com/RustSec/rustsec/tree/main/cargo-audit) across the entire workspace. The command installs `cargo-audit` if needed and reports vulnerable or unmaintained dependencies surfaced via the RustSec advisory database. Treat the warnings as action items when possible, and file follow-up issues if immediate remediation is not feasible.

## Gemstone

Cross platform Rust library for iOS and Android with native async networking support.

### iOS

Download `Gemstone-spm.tar.bz2` from the [releases](https://github.com/gemwalletcom/core/releases) page.

Unzip and add it to your project as a local Swift Package.

### Android

Build the Gemstone Android AAR from source and publish it to the local Maven cache:

```bash
just gemstone build-android
```

Then consume it from `mavenLocal()` in your Android project.

# Contributing

We welcome contributions! To get started:

- Look for issues with the `help wanted` labels.
- Fork the repository.
- Create a new branch (feature-xyz).
- Commit your changes and push.
- Open a Pull Request.

# License

This project is licensed under the [MIT](./LICENSE) License.

# Community & Support

- 💬 Join our [Discord](https://discord.com/invite/aWkq5sj7SY) or [Telegram](https://t.me/gemwallet_developers)
- 📖 Read the [Docs](https://docs.gemwallet.com/)
- 🐦 Follow us on [X](https://x.com/gemwallet)

Made with ❤️ by the Gem Wallet community.
