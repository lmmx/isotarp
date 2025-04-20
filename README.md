# Isotarp

Isotarp is a tool to help Rust developers identify which tests provide unique code coverage. Think of it as an "isotope analyser" for cargo-tarpaulin.

## Motivation

When writing tests to improve code coverage, it's valuable to know:

1. Which lines of code are covered by each test
2. Which tests provide unique coverage (covering lines that no other test covers)
3. Which tests might be redundant from a coverage perspective

Cargo-tarpaulin is excellent for measuring overall coverage, but doesn't provide this test-specific information. Isotarp fills this gap, by doing the slightly awkward dance of enumerating all the tests, running cargo tarpaulin for each of them, filtering the results so as to not produce tons of JSON in the process.

## Installation

Ensure you have `cargo-tarpaulin` installed first:

```bash
cargo install cargo-tarpaulin
```

Then install Isotarp:

```bash
cargo install isotarp
```

Or from source:

```bash
git clone https://github.com/yourusername/isotarp.git
cd isotarp
cargo install --path .
```

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

## How It Works

Isotarp runs each test individually through cargo-tarpaulin to generate coverage data, then:

1. Collects which lines are covered by each test
2. Identifies lines uniquely covered by a specific test
3. Generates a comprehensive report showing which tests provide unique coverage

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

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
