use std::fmt::Write;

pub(crate) fn dump_hex(buffer: &[u8]) -> String {
    let mut hex_bytes = String::with_capacity(2 * buffer.len());
    for (i, &byte) in buffer.iter().enumerate() {
        if i % 16 == 0 {
            std::write!(hex_bytes, "\n").expect("Dumping hex data failed");
        }
        std::write!(hex_bytes, "{:02X} ", byte).expect("Dumping hex data failed");
    }
    hex_bytes
}
