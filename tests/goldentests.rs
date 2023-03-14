use goldentests::{ TestConfig, TestResult };

#[test]
fn run_golden_tests() -> TestResult<()> {
    let mut config = TestConfig::new("target/debug/list", "input", "-- ")?;
    // config.overwrite_tests = true;
    config.run_tests()
}
