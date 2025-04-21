pub fn foo() -> i32 {
    println!("This is foo function");
    42
}

pub fn bar() -> &'static str {
    println!("This is bar function");
    "bar result"
}
