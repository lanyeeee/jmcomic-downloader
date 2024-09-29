pub fn filename_filter(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                    '\\' | '/' => ' ',
                    ':' => '：',
                    '*' => '⭐',
                    '?' => '？',
                    '"' => '\'',
                    '<' => '《',
                    '>' => '》',
                    '|' => '丨',
                    '.' => '·',
                    _ => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
}

// 计算MD5哈希并返回十六进制字符串
pub fn md5_hex(data: &str) -> String {
        format!("{:x}", md5::compute(data))
}
