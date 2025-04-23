use wildmatch::WildMatch;

// Categorize patterns by type for more efficient processing
fn categorize_patterns(
    patterns: &[String],
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut exact_patterns = Vec::new();
    let mut exact_names = Vec::new();
    let mut path_wildcard_patterns = Vec::new();
    let mut name_wildcard_patterns = Vec::new();

    for pattern in patterns {
        if pattern.contains('*') || pattern.contains('?') {
            if pattern.contains("::") {
                path_wildcard_patterns.push(pattern.clone());
            } else {
                name_wildcard_patterns.push(pattern.clone());
            }
        } else {
            if pattern.contains("::") {
                exact_patterns.push(pattern.clone());
            } else {
                exact_names.push(pattern.clone());
            }
        }
    }

    (
        exact_patterns,
        exact_names,
        path_wildcard_patterns,
        name_wildcard_patterns,
    )
}

// Main utility function to resolve test patterns to test names
pub fn resolve_test_patterns(
    available_tests: &[String],
    patterns: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut selected_tests = Vec::new();
    let mut invalid_patterns = Vec::new();

    // Categorize patterns first
    let (exact_patterns, exact_names, path_wildcard_patterns, name_wildcard_patterns) =
        categorize_patterns(patterns);

    // Process exact matches first (most efficient)
    for pattern in exact_patterns {
        if available_tests.contains(&pattern) {
            selected_tests.push(pattern.clone());
        } else {
            invalid_patterns.push(pattern);
        }
    }

    // Process exact name matches
    for pattern in exact_names {
        let matches = match_test_by_name(available_tests, &pattern);
        if !matches.is_empty() {
            selected_tests.extend(matches);
        } else {
            invalid_patterns.push(pattern);
        }
    }

    // Process path wildcard patterns
    for pattern in path_wildcard_patterns {
        let matches = match_test_by_path(available_tests, &pattern);
        if !matches.is_empty() {
            selected_tests.extend(matches);
        } else {
            invalid_patterns.push(pattern);
        }
    }

    // Process name wildcard patterns
    for pattern in name_wildcard_patterns {
        let matches = match_test_by_name(available_tests, &pattern);
        if !matches.is_empty() {
            selected_tests.extend(matches);
        } else {
            invalid_patterns.push(pattern);
        }
    }

    // Remove duplicates
    selected_tests.sort();
    selected_tests.dedup();

    (selected_tests, invalid_patterns)
}

// Match tests by their full path (including ::)
fn match_test_by_path(available_tests: &[String], pattern: &str) -> Vec<String> {
    let wildcard = WildMatch::new(pattern);
    available_tests
        .iter()
        .filter(|test| wildcard.matches(test))
        .cloned()
        .collect()
}

// Match tests by just their name (part after the last ::)
fn match_test_by_name(available_tests: &[String], pattern: &str) -> Vec<String> {
    let wildcard = WildMatch::new(pattern);
    available_tests
        .iter()
        .filter(|test| {
            if let Some(test_name) = test.split("::").last() {
                wildcard.matches(test_name)
            } else {
                false
            }
        })
        .cloned()
        .collect()
}
