use std::str;

use std::io::{Cursor, Read, Seek, SeekFrom};
pub fn read_cursor(mut cursor: Cursor<Vec<u8>>) -> String {
    let mut bytes = Vec::new();
    cursor.seek(SeekFrom::Start(0)).unwrap();
    cursor.read_to_end(&mut bytes).unwrap();

    str::from_utf8(&bytes).unwrap().to_string()
}
