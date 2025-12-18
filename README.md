# medley

**A collection of high-quality, generic Rust utility modules designed for maximum convenience and minimal dependency footprint.**

[![crates.io](https://img.shields.io/crates/v/medley.svg)](https://crates.io/crates/medley)
[![License](https://img.shields.io/crates/l/medley.svg)](https://github.com/Envelica/medley/blob/main/LICENSE-MIT)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Envelica/medley/.github/workflows/rust.yml?branch=main)](https://github.com/Envelica/medley/actions)

## ⚠️ Stability Warning: Under Heavy Development

**The medley crate is currently under active, heavy development and is NOT ready for production use.**

We are working toward a stable **1.0.0** release, but until then:
* The API (**function names, module structure, and signatures**) may change drastically without prior warning.
* Functionality is actively being added, tested, and refactored.

We encourage review, testing, and feedback, but please do not rely on this crate in mission-critical applications until it reaches **1.0.0**.

## About

The purpose of medley is to aggregate various small, frequently needed utility features—from complex data structure extensions to simple trait implementations—into a single, well-organized crate. This allows developers to add one dependency (medley) instead of cluttering their Cargo.toml with multiple niche crates, thereby **keeping the overall dependency tree small and manageable.**

### Why medley?

* **Small Depencency Footprint:** Aggregates functionality with a strong focus on minimal dependencies.
* **Ergonomics:** Provides intuitive, easy-to-use APIs for common development needs.
* **Modular:** Modules are designed to be entirely independent, preventing feature bloat.

## Usage (Future)

Once stable, you will be able to add medley to your project using Cargo:

`	oml
[dependencies]
medley =  0.1 # Use the latest stable version when released
`

## Examples

Run the expression pull-parser example:

`
cargo run --example expr_pull
`

Run the streaming CSV pull-parser example:

`
cargo run --example csv_pull
`