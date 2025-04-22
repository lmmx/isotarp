use isotarp::resolve::resolve_test_patterns;

#[test]
fn test_exact_match() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ];
    let patterns = vec!["tests::test_foo".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert_eq!(matched.len(), 1);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_non_prefixed_exact_match() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ];
    let patterns = vec!["test_foo".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify - this should match against the suffix
    assert_eq!(matched.len(), 1);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_wildcard_all_tests() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ];
    let patterns = vec!["test_*".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert_eq!(matched.len(), 2);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(matched.contains(&"tests::test_not_bar".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_wildcard_module_prefix() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
        "other::test_something".to_string(),
    ];
    let patterns = vec!["tests::*".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert_eq!(matched.len(), 2);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(matched.contains(&"tests::test_not_bar".to_string()));
    assert!(!matched.contains(&"other::test_something".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_wildcard_specific_prefix() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ];
    let patterns = vec!["tests::test_f*".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert_eq!(matched.len(), 1);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(!matched.contains(&"tests::test_not_bar".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_wildcard_non_prefixed_specific() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ];
    let patterns = vec!["test_f*".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert_eq!(matched.len(), 1);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(!matched.contains(&"tests::test_not_bar".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_no_match() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ];
    let patterns = vec!["nonexistent".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert!(matched.is_empty());
    assert_eq!(unmatched.len(), 1);
    assert!(unmatched.contains(&"nonexistent".to_string()));
}

#[test]
fn test_multiple_patterns() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
        "other::test_something".to_string(),
    ];
    let patterns = vec!["test_f*".to_string(), "other::*".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert_eq!(matched.len(), 2);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(matched.contains(&"other::test_something".to_string()));
    assert!(!matched.contains(&"tests::test_not_bar".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_duplicate_matches() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_not_bar".to_string(),
    ];
    let patterns = vec!["test_f*".to_string(), "tests::test_foo".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify - should deduplicate matches
    assert_eq!(matched.len(), 1);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(unmatched.is_empty());
}

#[test]
fn test_question_mark_wildcard() {
    // Setup test data
    let available_tests = vec![
        "tests::test_foo".to_string(),
        "tests::test_fo".to_string(),
        "tests::test_food".to_string(),
    ];
    let patterns = vec!["test_fo?".to_string()];

    // Execute
    let (matched, unmatched) = resolve_test_patterns(&available_tests, &patterns);

    // Verify
    assert_eq!(matched.len(), 1);
    assert!(matched.contains(&"tests::test_foo".to_string()));
    assert!(!matched.contains(&"tests::test_fo".to_string()));
    assert!(!matched.contains(&"tests::test_food".to_string()));
    assert!(unmatched.is_empty());
}
