use crate::parser::parse_object;

pub(crate) fn short_object_bounds(bytes: &[u8]) -> Option<(usize, usize)> {
    let bytes = &bytes[..bytes.len().min(4096)];
    fn white(byte: u8) -> bool {
        matches!(byte, 9 | 10 | 12 | 13 | 32)
    }
    fn delimiter(byte: u8) -> bool {
        white(byte)
            || matches!(
                byte,
                b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
            )
    }
    let mut i = 0;
    while i < bytes.len() && white(bytes[i]) {
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    std::str::from_utf8(&bytes[start..i])
        .ok()?
        .parse::<u32>()
        .ok()?;
    if i == bytes.len() || !white(bytes[i]) {
        return None;
    }
    while i < bytes.len() && white(bytes[i]) {
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    std::str::from_utf8(&bytes[start..i])
        .ok()?
        .parse::<u16>()
        .ok()?;
    if i == bytes.len() || !white(bytes[i]) {
        return None;
    }
    while i < bytes.len() && white(bytes[i]) {
        i += 1;
    }
    if bytes.get(i..i + 3)? != b"obj" {
        return None;
    }
    let header_end = i + 3;
    if !delimiter(*bytes.get(header_end)?) {
        return None;
    }
    let body = &bytes[header_end..];
    let (remaining, object) = parse_object(body).ok()?;
    if matches!(object, crate::object::Object::Stream { .. }) {
        return None;
    }
    let object_end = bytes.len() - remaining.len();
    let mut tail = remaining;
    loop {
        while tail.first().is_some_and(|b| white(*b)) {
            tail = &tail[1..];
        }
        if tail.first() == Some(&b'%') {
            let end = tail.iter().position(|b| matches!(*b, b'\n' | b'\r'))?;
            tail = &tail[end..];
        } else {
            break;
        }
    }
    if !tail.starts_with(b"endobj") {
        return None;
    }
    if !tail.get(6).is_some_and(|b| delimiter(*b)) {
        return None;
    }
    Some((header_end, object_end))
}
