fn to_hex(n: u16) -> char {
    match n {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        _ => panic!("{n} is not a hex digit (0..16)"),
    }
}

pub(crate) fn escape_string(input: &str, out: &mut String) {
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x20'..='\x7e' => out.push(ch),
            _ => {
                let mut buf = [0; 2];
                for &mut code_unit in ch.encode_utf16(&mut buf) {
                    out.push_str("\\u");
                    out.push(to_hex(code_unit / 0x1000));
                    out.push(to_hex(code_unit % 0x1000 / 0x100));
                    out.push(to_hex(code_unit % 0x100 / 0x10));
                    out.push(to_hex(code_unit % 0x10));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn test_json_str(input: &str, expected: &str) {
        let mut out = String::new();
        escape_string(input, &mut out);
        assert_eq!(out, expected);
    }

    #[test]
    fn json_string_encoding() {
        test_json_str("abc", "abc");
        test_json_str("fancy string\n", "fancy string\\n");
        test_json_str("\n\t\r\0\\\'\"", "\\n\\t\\r\\u0000\\\\'\\\"");
        test_json_str("\u{1d54a}", "\\ud835\\udd4a");
    }
}
