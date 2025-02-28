//! This module contains util functions

/// Removes a UTF‑8 Byte Order Mark (BOM) from the beginning of a byte slice if present.
///
/// The UTF‑8 BOM is the byte sequence `[0xEF, 0xBB, 0xBF]`. This function checks whether
/// the provided slice starts with these bytes and, if so, returns a subslice without them.
/// Otherwise, it returns the original slice.
pub fn strip_utf8_bom(data: &[u8]) -> &[u8] {
    const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
    if data.starts_with(BOM) {
        &data[BOM.len()..]
    } else {
        data
    }
}
