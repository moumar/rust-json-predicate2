use json_predicate::Predicate;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
struct TestCase {
    message: String,
    predicate: Predicate,
    expect: bool,
}

#[test]
fn test_predicates() -> Result<(), Box<dyn std::error::Error>> {
    let input: Value = serde_json::from_str(&std::fs::read_to_string("tests/input.json")?)?;
    let test_cases: Vec<TestCase> =
        serde_json::from_str(&std::fs::read_to_string("tests/test-cases.json")?)?;

    for test_case in test_cases {
        let result = test_case.predicate.test(&input);
        assert_eq!(result, test_case.expect, "{}", test_case.message);
    }
    // println!("{test_cases:#?}");
    // assert!(false);
    Ok(())
}
