mod util;
use lox::vm::InterpretError;
use rstest::*;
use util::assert_interpreter_output;

const RETURN_FALSE: &str = "false\n";
const RETURN_TRUE: &str = "true\n";

#[rstest]
#[case::add("print 1 + 1;", "2\n", "", Result::Ok(()))]
#[case::divide("print 2/3;", "0.6666666666666666\n", "", Result::Ok(()))]
#[case::not("print !true;", RETURN_FALSE, "", Result::Ok(()))]
#[case::equal_false("print 1 == 2;", RETURN_FALSE, "", Result::Ok(()))]
#[case::equal_true("print 1 == 1;", RETURN_TRUE, "", Result::Ok(()))]
#[case::string_eq("print \"asdf\n\" == \"asdf\n\";", RETURN_TRUE, "", Result::Ok(()))]
#[case::string_neq("print \"xyz\" == \"yzx\";", RETURN_FALSE, "", Result::Ok(()))]
#[case::string_concat("print \"a\" + \"b\" == \"ab\";", RETURN_TRUE, "", Result::Ok(()))]
#[case::global("var GLOB = 1; print GLOB;", "1\n", "", Result::Ok(()))]
#[case::global_default("var GLOB; print GLOB;", "nil\n", "", Result::Ok(()))]
#[case::global_uninit("print UNINIT;", "", "", Result::Err(InterpretError::RuntimeError(
    "Undefined variable 'UNINIT'.\n[line 1] in script\n".to_string())))]
#[case::global_default(
"var A = 3;\
var B = 5;\
A = A + B;
print A;", "8\n", "", Result::Ok(()))]
fn interpreter(
    #[case] input: &str,
    #[case] expected_output: &str,
    #[case] expected_error: &str,
    #[case] expected_result: Result<(), InterpretError>,
) {
    assert_interpreter_output(input, expected_output, expected_error, expected_result)
}
