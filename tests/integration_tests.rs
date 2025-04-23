use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_coverage_analysis_demo_lib() {
    // First ensure we're in the right directory
    let test_fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    assert!(
        test_fixtures_dir.exists(),
        "Test fixtures directory doesn't exist"
    );

    let demo_lib_dir = test_fixtures_dir.join("demolib");
    assert!(demo_lib_dir.exists(), "Demo lib directory doesn't exist");

    // Build the demo library first to ensure it's working
    let build_output = Command::new("cargo")
        .current_dir(&demo_lib_dir)
        .args(["test", "--no-run"])
        .output()
        .expect("Failed to build demo library");

    assert!(
        build_output.status.success(),
        "Failed to build demo library: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Run the isotarp analysis command
    let output_dir = demo_lib_dir.join("isotarp-output");
    let report_file = demo_lib_dir.join("isotarp-analysis.json");

    // Remove any existing output from previous test runs
    let _ = fs::remove_dir_all(&output_dir);
    let _ = fs::remove_file(&report_file);

    // Get the path to the compiled isotarp binary
    let isotarp_bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/isotarp");

    // Run the isotarp command with the current working directory set to the demo lib directory
    let analyze_output = Command::new(isotarp_bin)
        .current_dir(&demo_lib_dir) // Set the working directory
        .args([
            "analyze",
            "-p",
            "demolib",
            "-o",
            "isotarp-output",
            "-r",
            "isotarp-analysis.json",
        ])
        .output()
        .expect("Failed to run isotarp analyze");

    // Print the output to debug
    println!(
        "Command stdout: {}",
        String::from_utf8_lossy(&analyze_output.stdout)
    );
    println!(
        "Command stderr: {}",
        String::from_utf8_lossy(&analyze_output.stderr)
    );

    assert!(
        analyze_output.status.success(),
        "Isotarp analysis failed: {}",
        String::from_utf8_lossy(&analyze_output.stderr)
    );

    // Read and parse the report
    assert!(report_file.exists(), "Report file wasn't created");
    let report_content = fs::read_to_string(&report_file).expect("Failed to read report file");

    let report: serde_json::Value =
        serde_json::from_str(&report_content).expect("Failed to parse report JSON");

    // Verify that test_foo has unique coverage and test_not_bar doesn't
    let tests = report["tests"]
        .as_object()
        .expect("tests not found or not an object");

    // Find test_foo
    let test_foo = tests
        .get("tests::test_foo")
        .expect("test_foo not found in report");
    let test_foo_unique = test_foo["unique_covered_lines"]
        .as_u64()
        .expect("unique_covered_lines not found");
    assert!(test_foo_unique > 0, "test_foo should have unique coverage");

    // Find test_not_bar
    let test_not_bar = tests
        .get("tests::test_not_bar")
        .expect("test_not_bar not found in report");
    let test_not_bar_unique = test_not_bar["unique_covered_lines"]
        .as_u64()
        .expect("unique_covered_lines not found");
    assert_eq!(
        test_not_bar_unique, 0,
        "test_not_bar should have NO unique coverage"
    );

    println!(
        "Test passed! test_foo has unique coverage: {}, test_not_bar has unique coverage: {}",
        test_foo_unique, test_not_bar_unique
    );
}
