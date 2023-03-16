use lox::vm::interpret;
use std::io::Cursor;

mod util;

#[test]
fn add() {
    let mut out_cursor = Cursor::new(Vec::new());
    let mut err_cursor = Cursor::new(Vec::new());

    interpret("1 + 1", &mut out_cursor, &mut err_cursor).unwrap();

    assert_eq!(util::read_cursor(out_cursor), "2\n");
    assert_eq!(util::read_cursor(err_cursor), "");
}

#[test]
fn not() {
    let mut out_cursor = Cursor::new(Vec::new());
    let mut err_cursor = Cursor::new(Vec::new());

    interpret("!true", &mut out_cursor, &mut err_cursor).unwrap();

    assert_eq!(util::read_cursor(out_cursor), "false\n");
    assert_eq!(util::read_cursor(err_cursor), "");
}
