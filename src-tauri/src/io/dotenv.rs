//! `.env` dosyaları.
//!
//! `.env` biçiminin tek bir standardı yoktur; yaygın varyasyonlar desteklenir:
//! `export ` öneki, `#` yorumları, boş satırlar, tek tırnak (değişmez), çift tırnak
//! (`\n`, `\t`, `\"`, `\\` kaçışları ve çok satırlı değerler), tırnaksız değerlerde ` #` satır içi yorum.
//!
//! **Hiçbir şey genişletilmez veya çalıştırılmaz.** `$VAR`, `${VAR}` ve `$(komut)` biçimleri
//! olduğu gibi metin olarak kalır; ayrıştırıcı bunlar için uyarı üretir.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub name: String,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Warning {
    pub line: usize,
    /// i18n: `importer.warn.<code>`
    pub code: &'static str,
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub entries: Vec<Entry>,
    pub warnings: Vec<Warning>,
}

fn unescape_double(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Kapanmamış bir çift tırnağın sonunu (kaçışları hesaba katarak) bulur.
fn find_closing_double(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(i);
        }
    }
    None
}

pub fn parse(text: &str) -> Parsed {
    let mut out = Parsed::default();
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line_no = i + 1;
        let raw = lines[i];
        i += 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line
            .strip_prefix("export ")
            .map(str::trim_start)
            .unwrap_or(line);
        let Some(eq) = line.find('=') else {
            out.warnings.push(Warning {
                line: line_no,
                code: "noEquals",
            });
            continue;
        };
        let name = line[..eq].trim().to_string();
        let rest = line[eq + 1..].trim_start();
        let value = if let Some(body) = rest.strip_prefix('"') {
            // Çift tırnak: kapanana kadar sonraki satırları da al.
            let mut buf = body.to_string();
            loop {
                if let Some(end) = find_closing_double(&buf) {
                    let tail = buf[end + 1..].trim();
                    if !tail.is_empty() && !tail.starts_with('#') {
                        out.warnings.push(Warning {
                            line: line_no,
                            code: "trailingContent",
                        });
                    }
                    break unescape_double(&buf[..end]);
                }
                if i >= lines.len() {
                    out.warnings.push(Warning {
                        line: line_no,
                        code: "unterminatedQuote",
                    });
                    break unescape_double(&buf);
                }
                buf.push('\n');
                buf.push_str(lines[i]);
                i += 1;
            }
        } else if let Some(body) = rest.strip_prefix('\'') {
            match body.find('\'') {
                Some(end) => body[..end].to_string(),
                None => {
                    out.warnings.push(Warning {
                        line: line_no,
                        code: "unterminatedQuote",
                    });
                    body.to_string()
                }
            }
        } else {
            // Tırnaksız: " #" sonrası yorumdur.
            let v = match rest.find(" #") {
                Some(p) => &rest[..p],
                None => rest,
            };
            v.trim_end().to_string()
        };
        if value.contains("$(") || value.contains('`') {
            out.warnings.push(Warning {
                line: line_no,
                code: "commandSubstitution",
            });
        } else if value.contains("${")
            || value
                .split('$')
                .skip(1)
                .any(|s| s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_'))
        {
            out.warnings.push(Warning {
                line: line_no,
                code: "variableReference",
            });
        }
        out.entries.push(Entry {
            name,
            value,
            line: line_no,
        });
    }
    out
}

fn needs_quotes(v: &str) -> bool {
    v.is_empty()
        || v.chars()
            .any(|c| c.is_whitespace() || matches!(c, '#' | '"' | '\'' | '\\' | '$' | '`' | '='))
}

pub fn quote(v: &str) -> String {
    if !needs_quotes(v) {
        return v.to_string();
    }
    let mut s = String::with_capacity(v.len() + 2);
    s.push('"');
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

pub fn write(entries: &[(String, String)], header: &str) -> String {
    let mut out = String::new();
    for l in header.lines() {
        out.push_str("# ");
        out.push_str(l);
        out.push('\n');
    }
    for (k, v) in entries {
        out.push_str(k);
        out.push('=');
        out.push_str(&quote(v));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_variations() {
        let text = "\u{feff}# yorum\n\nexport A=1\nB = two words # yorum\nC='lit $HOME #x'\nD=\"x\\ny \\\"q\\\"\"\nE=\"çok\nsatırlı\"\nF=\nG=a=b\nINVALID LINE\nH=$(rm -rf /)\nI=${HOME}/x\n";
        let p = parse(text);
        let get = |n: &str| {
            p.entries
                .iter()
                .find(|e| e.name == n)
                .map(|e| e.value.clone())
        };
        assert_eq!(get("A").as_deref(), Some("1"));
        assert_eq!(get("B").as_deref(), Some("two words"));
        assert_eq!(
            get("C").as_deref(),
            Some("lit $HOME #x"),
            "tek tırnak değişmezdir"
        );
        assert_eq!(get("D").as_deref(), Some("x\ny \"q\""));
        assert_eq!(get("E").as_deref(), Some("çok\nsatırlı"));
        assert_eq!(get("F").as_deref(), Some(""));
        assert_eq!(get("G").as_deref(), Some("a=b"));
        assert_eq!(
            get("H").as_deref(),
            Some("$(rm -rf /)"),
            "komut çalıştırılmaz, metin olarak kalır"
        );
        assert_eq!(
            get("I").as_deref(),
            Some("${HOME}/x"),
            "genişletme yapılmaz"
        );
        let codes: Vec<&str> = p.warnings.iter().map(|w| w.code).collect();
        assert!(codes.contains(&"noEquals"));
        assert!(codes.contains(&"commandSubstitution"));
        assert!(codes.contains(&"variableReference"));
    }

    #[test]
    fn unterminated_quote_is_reported() {
        let p = parse("A=\"open\nB=2");
        assert_eq!(p.warnings[0].code, "unterminatedQuote");
    }

    #[test]
    fn write_parse_round_trip() {
        let entries = vec![
            ("A".to_string(), "simple".to_string()),
            ("B".to_string(), "with space # and hash".to_string()),
            ("C".to_string(), "line1\nline2 \"q\" \\ $X".to_string()),
            ("D".to_string(), String::new()),
            ("E".to_string(), "C:\\Program Files\\x".to_string()),
        ];
        let text = write(&entries, "Ivily Envora");
        let p = parse(&text);
        let back: Vec<(String, String)> =
            p.entries.into_iter().map(|e| (e.name, e.value)).collect();
        assert_eq!(back, entries);
    }
}
