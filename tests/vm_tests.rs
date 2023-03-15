use lox::vm::interpret;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::str;

#[test]
fn add() {
    let mut out_cursor = Cursor::new(Vec::new());
    let mut err_cursor = Cursor::new(Vec::new());

    interpret("1 + 1", &mut out_cursor, &mut err_cursor).unwrap();

    let mut out = Vec::new();
    out_cursor.seek(SeekFrom::Start(0)).unwrap();
    out_cursor.read_to_end(& mut out).unwrap();
    let mut err = Vec::new();
    err_cursor.seek(SeekFrom::Start(0)).unwrap();
    err_cursor.read_to_end(& mut err).unwrap();

    assert_eq!(str::from_utf8(&out).unwrap(), "2\n");
    assert_eq!(str::from_utf8(&err).unwrap(), "");
}
