#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/message-test.rs");
    t.pass("tests/unit-test.rs");
}
