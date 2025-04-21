pub mod functions;
pub use functions::*;

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
}
