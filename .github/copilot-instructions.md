# Repository guide

## Build, run, and checks

- Build: `cargo build` (release binaries use `cargo build --release`).
- Run locally: `cargo run -- "<description of the command to generate>"`.
- Run tests: `cargo test`. To run one test or a filtered set, use `cargo test <test_name_or_filter>`. There are currently no test cases in the repository.
- Check formatting: `cargo fmt --all -- --check`.
- Run Clippy: `cargo clippy --all-features --all-targets`.

The formatting and Clippy commands match `.github/workflows/lint.yml`. CI uses the nightly Rust toolchain with `rustfmt` and `clippy`.

## Architecture

This is a small Rust CLI with its argument parsing and command-generation workflow in `src/main.rs`, and environment-based settings and shell-history handling in `src/config.rs`.

`main` parses the prompt and `--force` option with Clap, builds a Bash-oriented prompt with a platform hint, and calls the OpenAI completions endpoint through a blocking Reqwest client. It displays the generated Bash with `bat`, asks for confirmation unless `--force` was given, then records approved commands in the detected Bash/Zsh history file and executes them with `bash -c`.

`Config::new` requires `OPENAI_API_KEY`; `OPENAI_API_BASE` overrides the default API base URL, and `SHELL` determines whether command history is written. Keep API/configuration handling in `config.rs` and the CLI flow in `main.rs`.

## Conventions

- CLI options and help text are derived from the Clap `Parser` struct in `main.rs`; update that definition rather than duplicating option parsing.
- The project enables Clippy's `all`, `pedantic`, and `nursery` warning groups at the crate root. Keep changes compatible with these lints.
- The release workflow builds platform-specific binaries for Windows, Linux (including musl), and macOS; preserve the `plz` binary name expected by those packaging steps.
