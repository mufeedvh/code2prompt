use code2prompt_core::util::strip_utf8_bom;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_utf8_bom_when_present() {
        let input = b"\xEF\xBB\xBFHello, world!";
        let expected = b"Hello, world!";
        let output = strip_utf8_bom(input);
        assert_eq!(
            output, expected,
            "BOM should be stripped from the beginning of the input."
        );
    }

    #[test]
    fn test_strip_utf8_bom_when_not_present() {
        let input = b"Hello, world!";
        let output = strip_utf8_bom(input);
        assert_eq!(
            output, input,
            "Input without a BOM should remain unchanged."
        );
    }

    #[test]
    fn test_strip_utf8_bom_empty_input() {
        let input = b"";
        let output = strip_utf8_bom(input);
        assert_eq!(
            output, input,
            "An empty input should return an empty output."
        );
    }

    #[test]
    fn test_strip_utf8_bom_only_bom() {
        let input = b"\xEF\xBB\xBF";
        let expected = b"";
        let output = strip_utf8_bom(input);
        assert_eq!(
            output, expected,
            "Input that is only a BOM should return an empty slice."
        );
    }
}
