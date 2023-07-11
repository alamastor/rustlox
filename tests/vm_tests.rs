mod util;
use rstest::*;
use util::assert_interpreter_output;

const RETURN_FALSE: &str = "false\n";
const RETURN_TRUE: &str = "true\n";

#[rstest]
#[case::add("1 + 1", "2\n", "")]
#[case::divide("2/3", "0.6666666666666666\n", "")]
#[case::not("!true", RETURN_FALSE, "")]
#[case::equal_false("1 == 2", RETURN_FALSE, "")]
#[case::equal_true("1 == 1", RETURN_TRUE, "")]
#[case::string_eq("\"asdf\n\" == \"asdf\n\"", RETURN_TRUE, "")]
#[case::string_neq("\"xyz\" == \"yzx\"", RETURN_FALSE, "")]
#[should_panic]
#[case::string_concat("\"a\" + \"b\" == \"ab\"", RETURN_TRUE, "")]
fn interpreter(#[case] input: &str, #[case] expected_output: &str, #[case] expected_error: &str) {
    assert_interpreter_output(input, expected_output, expected_error) 
}
