//! JSON as the Fixpt answer cache keeps it: read whatever Python's `json`
//! writes, and written as `json.dumps(d, indent=0, ensure_ascii=False)`
//! writes it, byte for byte, so that a file the Python scripts and these
//! tests both keep (`tests/fixpt-answers/`) does not change by who wrote it.

/// A JSON value; an object's members in their order.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(ms) => ms.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Json> {
        match self {
            Json::Obj(ms) => ms.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn arr(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn strs(&self) -> Vec<String> {
        self.arr()
            .map(|a| a.iter().filter_map(|x| x.str().map(String::from)).collect())
            .unwrap_or_default()
    }
}

/// `text` read; `Err` saying where it is not JSON.
pub fn parse(text: &str) -> Result<Json, String> {
    let mut p = Parser {
        s: text.as_bytes(),
        i: 0,
    };
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("JSON: more after the value, at {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\n' | b'\r' | b'\t') {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.ws();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("JSON: {:?} expected at {}", c as char, self.i))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.s.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut ms = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(Json::Obj(ms));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.eat(b':')?;
                    ms.push((k, self.value()?));
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Json::Obj(ms));
                        }
                        _ => return Err(format!("JSON: , or }} expected at {}", self.i)),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut xs = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Json::Arr(xs));
                }
                loop {
                    xs.push(self.value()?);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Json::Arr(xs));
                        }
                        _ => return Err(format!("JSON: , or ] expected at {}", self.i)),
                    }
                }
            }
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(Json::Bool(true))
            }
            Some(b'f') if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(Json::Bool(false))
            }
            Some(b'n') if self.s[self.i..].starts_with(b"null") => {
                self.i += 4;
                Ok(Json::Null)
            }
            Some(c) if *c == b'-' || c.is_ascii_digit() => {
                let start = self.i;
                self.i += 1;
                while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.s[start..self.i]).unwrap();
                t.parse()
                    .map(Json::Int)
                    .map_err(|e| format!("JSON: {t}: {e}"))
            }
            _ => Err(format!("JSON: a value expected at {}", self.i)),
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let t = self
            .s
            .get(self.i..self.i + 4)
            .ok_or("JSON: \\u cut short")?;
        self.i += 4;
        u32::from_str_radix(std::str::from_utf8(t).map_err(|e| e.to_string())?, 16)
            .map_err(|e| e.to_string())
    }

    fn string(&mut self) -> Result<String, String> {
        self.ws();
        if self.s.get(self.i) != Some(&b'"') {
            return Err(format!("JSON: a string expected at {}", self.i));
        }
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            match self.s.get(self.i) {
                None => return Err("JSON: a string not ended".into()),
                Some(b'"') => {
                    self.i += 1;
                    return String::from_utf8(out).map_err(|e| e.to_string());
                }
                Some(b'\\') => {
                    self.i += 1;
                    let c = *self.s.get(self.i).ok_or("JSON: an escape cut short")?;
                    self.i += 1;
                    let ch = match c {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let hi = self.hex4()?;
                            let code = if (0xd800..0xdc00).contains(&hi)
                                && self.s[self.i..].starts_with(b"\\u")
                            {
                                self.i += 2;
                                let lo = self.hex4()?;
                                0x10000 + ((hi - 0xd800) << 10) + (lo - 0xdc00)
                            } else {
                                hi
                            };
                            char::from_u32(code).unwrap_or('\u{fffd}')
                        }
                        _ => return Err(format!("JSON: an escape \\{}", c as char)),
                    };
                    let mut b = [0; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut b).as_bytes());
                }
                Some(&c) => {
                    out.push(c);
                    self.i += 1;
                }
            }
        }
    }
}

fn put_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// As Python's `json.dumps(v, indent=0, ensure_ascii=False)`.
pub fn dumps_indent0(v: &Json) -> String {
    let mut out = String::new();
    put(v, &mut out);
    out
}

fn put(v: &Json, out: &mut String) {
    match v {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Int(n) => out.push_str(&n.to_string()),
        Json::Str(s) => put_str(s, out),
        Json::Arr(xs) if xs.is_empty() => out.push_str("[]"),
        Json::Arr(xs) => {
            out.push_str("[\n");
            for (i, x) in xs.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                put(x, out);
            }
            out.push_str("\n]");
        }
        Json::Obj(ms) if ms.is_empty() => out.push_str("{}"),
        Json::Obj(ms) => {
            out.push_str("{\n");
            for (i, (k, x)) in ms.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                put_str(k, out);
                out.push_str(": ");
                put(x, out);
            }
            out.push_str("\n}");
        }
    }
}

/// Compact, as `json.dump` writes by default (`", "` and `": "`), for
/// files only read back.
pub fn dumps(v: &Json) -> String {
    match v {
        Json::Arr(xs) => format!("[{}]", xs.iter().map(dumps).collect::<Vec<_>>().join(", ")),
        Json::Obj(ms) => {
            let parts: Vec<String> = ms
                .iter()
                .map(|(k, x)| {
                    let mut s = String::new();
                    put_str(k, &mut s);
                    format!("{s}: {}", dumps(x))
                })
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        other => dumps_indent0(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_answer_kept_is_written_back_as_it_was() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixpt-answers");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            let text = std::fs::read_to_string(&p).unwrap();
            let v = parse(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            assert_eq!(dumps_indent0(&v) + "\n", text, "{}", p.display());
            n += 1;
        }
        assert!(n > 100);
    }

    #[test]
    fn escapes_read() {
        assert_eq!(parse(r#""aé😀\n""#).unwrap(), Json::Str("aé😀\n".into()));
    }
}
