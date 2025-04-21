## demolib

A demonstration of a library with:

- one function `foo` with 1 test covering it
  -  tested by `test_foo`
- one function `bar` with 0 test coverage
- one test function `test_no_bar` which doesn't cover any code

This is the unique situation where you find that running on the entire package (all tests) gives a
test that (\*should) reports "0 unique lines" when called on its own.

Note that a function will report 0 unique lines if called in an analysis with another test that
covers the same lines.

```shell
louis 🌟 ~/lab/isotarp/demolib $ tree src/
src/
└── lib.rs

1 directory, 1 file
```

The file contains:

```rust
louis 🌟 ~/lab/isotarp/demolib $ cat src/lib.rs
pub fn foo() -> i32 {
    println!("This is foo function");
    42
}

pub fn bar() -> &'static str {
    println!("This is bar function");
    "bar result"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        assert_eq!(foo(), 42);
    }

    #[test]
    fn test_not_bar() {
        println!("Hello from test_not_bar");
        // This test doesn't call bar() at all
        assert!(true);
    }
```

Here is the analysis of the entire package

```shell
isotarp analyze -p demolib
```

```
No specific tests provided, analyzing all tests...
Analyzing 2 tests in package 'demolib'
Cleaning and building package...
     Removed 41 files, 6.9MiB total
   Compiling demolib v0.1.0 (/home/louis/lab/isotarp/demolib)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
Preparing target directories for parallel execution...
Preparing target directory for test: tests::test_foo
Preparing target directory for test: tests::test_not_bar
Running coverage for test: tests::test_foo
Running coverage for test: tests::test_not_bar
Cleaning up temporary target directories...
Analysis complete! Results saved to isotarp-analysis.json

Tests ranked by unique line coverage:
  tests::test_foo: 2 unique lines (100.0% of 2 total covered lines)
  tests::test_not_bar: 0 unique lines (0.0% of 0 total covered lines)

Tests with NO unique coverage:
  tests::test_not_bar
Cleaning up temporary target directories...
```

Then `test_foo`

```shell
isotarp analyze -p demolib -t test_foo
```

```
Analyzing 1 tests in package 'demolib'
Cleaning and building package...
     Removed 28 files, 6.8MiB total
   Compiling demolib v0.1.0 (/home/louis/lab/isotarp/demolib)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
Preparing target directories for parallel execution...
Preparing target directory for test: test_foo
Running coverage for test: test_foo
Cleaning up temporary target directories...
Analysis complete! Results saved to isotarp-analysis.json

Tests ranked by unique line coverage:
  test_foo: 2 unique lines (100.0% of 2 total covered lines)
Cleaning up temporary target directories...
```

Then `test_not_bar`

```shell
isotarp analyze -p demolib -t test_not_bar
```

```
Analyzing 1 tests in package 'demolib'
Cleaning and building package...
     Removed 28 files, 6.8MiB total
   Compiling demolib v0.1.0 (/home/louis/lab/isotarp/demolib)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
Preparing target directories for parallel execution...
Preparing target directory for test: test_not_bar
Running coverage for test: test_not_bar
Cleaning up temporary target directories...
Analysis complete! Results saved to isotarp-analysis.json

Tests ranked by unique line coverage:
  test_not_bar: 0 unique lines (0.0% of 0 total covered lines)

Tests with NO unique coverage:
  test_not_bar
Cleaning up temporary target directories...
```
