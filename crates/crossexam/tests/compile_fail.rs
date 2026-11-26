#[test]
fn verified_cannot_be_constructed_outside_the_kernel() {
    trybuild::TestCases::new().compile_fail("tests/ui/forge_verified.rs");
}
