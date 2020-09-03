// For now, this is a very generic library.

use core::fmt::Write;

pub fn format_hex(bytes: &[u8]) -> String {
    let mut buffer = String::with_capacity(2 * bytes.len());

    for byte in bytes {
        write!(buffer, "{:02X}", byte).unwrap();
    }

    buffer
}
