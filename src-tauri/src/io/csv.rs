//! CSV (RFC 4180) okuma/yazma — yalnızca değişken listeleri için.
//!
//! Sütunlar: `name,value,scope,kind`. Elektronik tablo programlarında formül olarak
//! yorumlanmasın diye `=`, `+`, `-`, `@`, sekme veya satır başıyla başlayan hücrelerin başına
//! dışa aktarmada `'` eklenir; içe aktarmada bu önek yalnızca bu karakterlerden birinin önündeyse kaldırılır.

use crate::error::{AppError, AppResult};

const DANGEROUS: &[char] = &['=', '+', '-', '@', '\t', '\r'];

pub fn guard(cell: &str) -> String {
    if cell.starts_with(DANGEROUS) {
        format!("'{cell}")
    } else {
        cell.to_string()
    }
}

pub fn unguard(cell: &str) -> String {
    match cell.strip_prefix('\'') {
        Some(rest) if rest.starts_with(DANGEROUS) => rest.to_string(),
        _ => cell.to_string(),
    }
}

fn escape(cell: &str) -> String {
    if cell.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell.to_string()
    }
}

pub fn write(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(&header.join(","));
    out.push_str("\r\n");
    for r in rows {
        let cells: Vec<String> = r.iter().map(|c| escape(&guard(c))).collect();
        out.push_str(&cells.join(","));
        out.push_str("\r\n");
    }
    out
}

/// CSV metnini satırlara ayırır (tırnaklı alanlar, kaçışlı tırnaklar ve çok satırlı hücreler desteklenir).
pub fn parse(text: &str) -> AppResult<Vec<Vec<String>>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    cell.push('"');
                    chars.next();
                }
                '"' => quoted = false,
                c => cell.push(c),
            }
            continue;
        }
        match c {
            '"' if cell.is_empty() => quoted = true,
            ',' => row.push(unguard(&std::mem::take(&mut cell))),
            '\r' => {}
            '\n' => {
                row.push(unguard(&std::mem::take(&mut cell)));
                if !(row.len() == 1 && row[0].is_empty()) {
                    rows.push(std::mem::take(&mut row));
                } else {
                    row.clear();
                }
            }
            c => cell.push(c),
        }
    }
    if quoted {
        return Err(AppError::validation("invalid_csv", "kapanmamış tırnak"));
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(unguard(&cell));
        rows.push(row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_with_special_cells() {
        let rows = vec![
            vec!["A".into(), "a,b".into(), "user".into(), "string".into()],
            vec![
                "B".into(),
                "say \"hi\"\nnext".into(),
                "machine".into(),
                "expand".into(),
            ],
            vec![
                "C".into(),
                "=HYPERLINK(\"x\")".into(),
                "user".into(),
                "string".into(),
            ],
            vec!["D".into(), "'keep".into(), "user".into(), "string".into()],
            vec!["E".into(), "-5".into(), "user".into(), "string".into()],
        ];
        let text = write(&["name", "value", "scope", "kind"], &rows);
        assert!(
            text.contains("\"'=HYPERLINK(\"\"x\"\")\""),
            "formül koruması: {text}"
        );
        let back = parse(&text).unwrap();
        assert_eq!(back[0], vec!["name", "value", "scope", "kind"]);
        assert_eq!(&back[1..], &rows[..]);
    }

    #[test]
    fn rejects_unterminated_quote() {
        assert!(parse("a,\"b\n").is_err());
    }
}
