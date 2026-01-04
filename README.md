# ulidgen

Simple and lightweight ULID generator.

## Install

Download the binary for your OS and extract it in a directory within your PATH.

## Usage

Run the command with no arguments to generate a ULID at the system time or specify a time with `-t` or `--time`.

The specified time has to be either in UNIX Timestamp, RFC 3339 or "Date Only".

The UNIX Timestamp must be as seconds or milliseconds. Anything up to 10 digits will be considered seconds, anything with more than 10 and up to 13 digits will be considered milliseconds. Anything with 14 or more digits will be considered invalid.

The RFC 3339 accepts offsets and fractional seconds. For example: `ulidgen -t 2026-01-01T12:34:56.789-03:00` or `ulidgen -t 2026-01-01T15:34:56.789Z`.

The Date Only accepts dates without time in the YYYY-MM-DD format, and it will treat it as midnight UTC. Running `ulidgen -t 2026-01-01` is the same as running `ulidgen -t 2026-01-01T00:00:00Z`.

## Install from Source

With the Rust and Cargo environment set up, run `cargo build --release`. Copy "./target/release/ulidgen" (".\target\release\ulidgen.exe" in Windows) to a directory within your PATH.
