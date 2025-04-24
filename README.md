# Isotarp

[![Coverage Status](https://coveralls.io/repos/github/lmmx/isotarp/badge.svg?branch=master)](https://coveralls.io/github/lmmx/isotarp?branch=master)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/lmmx/isotarp/binaries.yml)](https://github.com/lmmx/isotarp/actions/workflows/binaries.yml)
[![crates.io](https://img.shields.io/crates/v/isotarp.svg)](https://crates.io/crates/isotarp)
[![documentation](https://docs.rs/isotarp/badge.svg)](https://docs.rs/isotarp)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/isotarp.svg)](./LICENSE)
[![pre-commit.ci status](https://results.pre-commit.ci/badge/github/lmmx/isotarp/master.svg)](https://results.pre-commit.ci/latest/github/lmmx/isotarp/master)

Isotarp is a tool to help Rust developers identify which tests provide unique code coverage.

## Motivation

When writing tests to improve code coverage, it's valuable to know:

1. Which lines of code are covered by each test
2. Which tests provide unique coverage (covering lines that no other test covers)
3. Which tests might be redundant from a coverage perspective

Cargo-tarpaulin is excellent for measuring overall coverage, but doesn't provide this test-specific information.

Isotarp fills this gap, by doing the slightly awkward dance of enumerating all the tests, running cargo tarpaulin for each of them, filtering the results so as to not produce tons of JSON in the process.

## Installation

Ensure you have `cargo-tarpaulin` installed first:

```bash
cargo install cargo-tarpaulin
```

Then install Isotarp:

```bash
cargo install isotarp
```

You can use `cargo binstall` for both if you prefer.

## Usage

### List all tests in a package

```bash
isotarp list -p your_package_name
```

### Analyze test coverage

Run analysis on all tests in a package:

```bash
isotarp analyze -p your_package_name
```

Or analyze specific tests:

```bash
isotarp analyze -p your_package_name -t test_name1 -t test_name2
```

You can customize output locations:

```bash
isotarp analyze -p your_package_name -o ./coverage -r coverage-report.json
```

### Target Directory Modes

Isotarp offers two modes for managing target directories during test execution:

```bash
# Default mode: creates separate target directories for each test (faster, more disk space)
isotarp analyze -p your_package_name --target-mode per

# Memory-efficient mode: reuses a single target directory (slower, less disk space)
isotarp analyze -p your_package_name --target-mode one
```

The `--target-mode` option accepts two values:

- `per` (default): Creates a separate target directory for each test, allowing parallel execution for faster results but requiring more disk space.
- `one`: Reuses a single target directory across tests sequentially, significantly reducing disk usage at the cost of some execution speed.

For large projects where target directories can grow to multiple GB, the `one` mode can reduce peak disk usage by 80-90% while only increasing execution time by about 50%.

## How It Works

Isotarp runs each test individually through cargo-tarpaulin to generate coverage data, then:

1. Collects which lines are covered by each test
2. Identifies lines uniquely covered by a specific test
3. Generates a comprehensive report showing which tests provide unique coverage

### Target Mode Implementation Details

- **Per Mode**: Creates individual copies of the target directory for each test, allowing parallel execution.
- **One Mode**: Uses a pipelined approach where:
  - A single target directory location is reused for all tests
  - The next test's directory is prepared in the background while the current test runs
  - Tests execute sequentially to avoid conflicts while minimizing wait time

## Output Format

The analysis produces a JSON file with detailed information about each test:
- Total lines covered
- Uniquely covered lines
- Files touched
- Line numbers for each uniquely covered line

## Example Output

Command-line summary:

```
Tests ranked by unique line coverage:
  integration::parsing::test_complex_case: 42 unique lines (58.3% of 72 total covered lines)
  integration::errors::test_invalid_input: 18 unique lines (45.0% of 40 total covered lines)
  unit::helpers::test_normalization: 5 unique lines (10.2% of 49 total covered lines)

Tests with NO unique coverage:
  unit::helpers::test_validation
```

## Performance Considerations

Choose the appropriate target mode based on your environment:

- Use `--target-mode per` (default) when:
  - You have plenty of disk space
  - You want the fastest possible execution
  - You're running on a system with multiple cores

- Use `--target-mode one` when:
  - Disk space is limited
  - Your project has a large target directory
  - You're willing to trade some speed for reduced disk usage

In testing, for a project with 6 tests generating 3GB peak disk usage in the default mode, switching to `--target-mode one` reduced peak usage to 0.5-0.7GB while increasing execution time by approximately 50%.

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
