use lox::vm::{interpret, InterpretError};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::str;

pub fn assert_interpreter_output(
    input: &str,
    expected_output: &str,
    expected_error: &str,
    expected_result: Result<(), InterpretError>,
) {
    let mut out_cursor = Cursor::new(Vec::new());
    let mut err_cursor = Cursor::new(Vec::new());

    assert_eq!(interpret(input, &mut out_cursor, &mut err_cursor), expected_result);

    assert_eq!(read_cursor(out_cursor), expected_output);
    assert_eq!(read_cursor(err_cursor), expected_error);
}

fn read_cursor(mut cursor: Cursor<Vec<u8>>) -> String {
    let mut bytes = Vec::new();
    cursor.seek(SeekFrom::Start(0)).unwrap();
    cursor.read_to_end(&mut bytes).unwrap();

    str::from_utf8(&bytes).unwrap().to_string()
}
