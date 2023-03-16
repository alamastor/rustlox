mod util;
use util::assert_interpreter_output;

#[test]
fn add() {
    assert_interpreter_output("1 + 1", "2\n", "");
}

#[test]
fn not() {
    assert_interpreter_output("!true", "false\n", "");
}
