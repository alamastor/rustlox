mod util;
use rstest::*;
use util::assert_interpreter_output;

#[rstest]
#[case::add("1 + 1", "2\n", "")]
#[case::divide("2/3", "0.6666666666666666\n", "")]
#[case::not("!true", "false\n", "")]
#[case::equal_false("1 == 2", "false\n", "")]
#[case::equal_true("1 == 1", "true\n", "")]
fn interpreter(#[case] input: &str, #[case] expected_output: &str, #[case] expected_error: &str) {
    assert_interpreter_output(input, expected_output, expected_error) 
}
