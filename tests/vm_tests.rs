mod util;
use rstest::*;
use util::assert_interpreter_output;

const RETURN_FALSE: &str = "false\n";
const RETURN_TRUE: &str = "true\n";

#[rstest]
#[case::add("print 1 + 1;", "2\n", "")]
#[case::divide("print 2/3;", "0.6666666666666666\n", "")]
#[case::not("print !true;", RETURN_FALSE, "")]
#[case::equal_false("print 1 == 2;", RETURN_FALSE, "")]
#[case::equal_true("print 1 == 1;", RETURN_TRUE, "")]
#[case::string_eq("print \"asdf\n\" == \"asdf\n\";", RETURN_TRUE, "")]
#[case::string_neq("print \"xyz\" == \"yzx\";", RETURN_FALSE, "")]
#[case::string_concat("print \"a\" + \"b\" == \"ab\";", RETURN_TRUE, "")]
#[case::define_global("var GLOB = 1;", "", "")]
#[case::define_global_uninit("var GLOB;", "", "")]
fn interpreter(#[case] input: &str, #[case] expected_output: &str, #[case] expected_error: &str) {
    assert_interpreter_output(input, expected_output, expected_error) 
}
