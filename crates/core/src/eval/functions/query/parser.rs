//! Parser for the QUERY pseudo-SQL query-language string (second argument to
//! `QUERY(data, query, [headers])`).
//!
//! Scope (see module doc in `mod.rs` for the full list of deferred clauses):
//! `select`, `where`, `group by`, `order by`, `limit`, `label`.
//!
//! Column identifiers use the `ColN` form (1-based, e.g. `Col1`, `Col2`),
//! matching Google Sheets' documented identifier scheme for a query run over
//! a literal array rather than a live cell range — which is what this
//! function always receives, since by the time `QUERY`'s eager handler runs,
//! `data` has already been evaluated down to a plain `Value::Array` with no
//! memory of originating cell coordinates. Spreadsheet-range queries would
//! additionally accept plain column letters (`A`, `B`, ...); that form is not
//! implemented here (see module doc).

use crate::types::{ErrorKind, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggFunc {
    Sum,
    Count,
    Avg,
    Max,
    Min,
}

impl AggFunc {
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "sum" => Some(AggFunc::Sum),
            "count" => Some(AggFunc::Count),
            "avg" | "average" => Some(AggFunc::Avg),
            "max" => Some(AggFunc::Max),
            "min" => Some(AggFunc::Min),
            _ => None,
        }
    }

    /// Lower-case token used to build the default output label, e.g. `sum Col2`.
    pub fn label_word(self) -> &'static str {
        match self {
            AggFunc::Sum => "sum",
            AggFunc::Count => "count",
            AggFunc::Avg => "avg",
            AggFunc::Max => "max",
            AggFunc::Min => "min",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectItem {
    /// 0-based data column index.
    Column(usize),
    /// Aggregate function over a 0-based data column index.
    Agg(AggFunc, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CondOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub col: usize,
    pub op: CondOp,
    /// Right-hand literal; `None` for `IS NULL` / `IS NOT NULL`.
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolJoin {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub conditions: Vec<Condition>,
    /// Uniform join operator across all conditions — mixed AND/OR precedence
    /// and parenthesised grouping are not supported (see module doc).
    pub join: BoolJoin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct OrderItem {
    pub col: usize,
    pub dir: SortDir,
}

#[derive(Debug, Clone)]
pub struct ParsedQuery {
    pub select: Vec<SelectItem>,
    pub where_clause: Option<WhereClause>,
    pub group_by: Vec<usize>,
    pub order_by: Vec<OrderItem>,
    pub limit: Option<usize>,
    /// `(select item this label targets, override text)`.
    pub labels: Vec<(SelectItem, String)>,
}

fn parse_error(msg: impl Into<String>) -> Value {
    Value::ErrorMsg(ErrorKind::Value, msg.into())
}

/// Lower-case only the ASCII letters of `s`, leaving every other byte (and
/// therefore every byte offset) untouched — so offsets found in the
/// lower-cased copy stay valid slice indices into the original string, even
/// when the original contains multi-byte UTF-8 text (e.g. quoted labels).
fn ascii_lower(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c })
        .collect()
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

const CLAUSE_KEYWORDS: [&str; 6] = ["group by", "order by", "select", "where", "limit", "label"];

/// Split `q` into `(keyword, content)` clause pairs, scanning for keywords at
/// word boundaries outside single-quoted string literals.
fn split_clauses(q: &str) -> Result<Vec<(&'static str, &str)>, Value> {
    let lower = ascii_lower(q);
    let bytes = lower.as_bytes();
    let n = bytes.len();
    let mut in_quote = false;
    let mut positions: Vec<(usize, &'static str)> = Vec::new();
    let mut i = 0;
    while i < n {
        let b = bytes[i];
        if b == b'\'' {
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        if !in_quote {
            let before_ok = i == 0 || !is_word_byte(bytes[i - 1]);
            if before_ok {
                let mut matched = None;
                for kw in CLAUSE_KEYWORDS {
                    let kwb = kw.as_bytes();
                    if bytes[i..].starts_with(kwb) {
                        let after = i + kwb.len();
                        let after_ok = after >= n || !is_word_byte(bytes[after]);
                        if after_ok {
                            matched = Some(kw);
                            break;
                        }
                    }
                }
                if let Some(kw) = matched {
                    positions.push((i, kw));
                    i += kw.len();
                    continue;
                }
            }
        }
        i += 1;
    }

    if positions.is_empty() {
        if q.trim().is_empty() {
            return Ok(vec![]);
        }
        return Err(parse_error(format!("Unable to parse query string for Function QUERY parameter 2: no clause keyword found near '{q}'")));
    }

    if !q[..positions[0].0].trim().is_empty() {
        return Err(parse_error(format!(
            "Unable to parse query string for Function QUERY parameter 2: unexpected text before '{}'",
            positions[0].1
        )));
    }

    let mut clauses = Vec::with_capacity(positions.len());
    for (idx, &(start, kw)) in positions.iter().enumerate() {
        let content_start = start + kw.len();
        let content_end = positions.get(idx + 1).map(|p| p.0).unwrap_or(q.len());
        clauses.push((kw, q[content_start..content_end].trim()));
    }
    Ok(clauses)
}

/// Quote-aware split of `s` on top-level occurrences of ASCII word `sep`
/// (e.g. `"and"`/`"or"`), case-insensitive, outside single-quoted strings.
fn split_on_word<'a>(s: &'a str, sep: &str) -> Vec<&'a str> {
    let lower = ascii_lower(s);
    let bytes = lower.as_bytes();
    let sepb = sep.as_bytes();
    let n = bytes.len();
    let mut in_quote = false;
    let mut parts = Vec::new();
    let mut last = 0;
    let mut i = 0;
    while i < n {
        let b = bytes[i];
        if b == b'\'' {
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        if !in_quote {
            let before_ok = i == 0 || !is_word_byte(bytes[i - 1]);
            if before_ok && bytes[i..].starts_with(sepb) {
                let after = i + sepb.len();
                let after_ok = after >= n || !is_word_byte(bytes[after]);
                if after_ok {
                    parts.push(s[last..i].trim());
                    last = after;
                    i = after;
                    continue;
                }
            }
        }
        i += 1;
    }
    parts.push(s[last..].trim());
    parts
}

/// Quote- and paren-aware split of `s` on top-level commas.
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_quote = false;
    let mut last = 0;
    for (i, c) in s.char_indices() {
        match c {
            '\'' => in_quote = !in_quote,
            '(' if !in_quote => depth += 1,
            ')' if !in_quote => depth -= 1,
            ',' if !in_quote && depth == 0 => {
                parts.push(s[last..i].trim());
                last = i + 1;
            }
            _ => {}
        }
    }
    parts.push(s[last..].trim());
    parts
}

/// Parse a `ColN` identifier (case-insensitive) into a 0-based column index.
fn parse_col_ref(s: &str) -> Option<usize> {
    let s = s.trim();
    let lower = s.to_ascii_lowercase();
    let digits = lower.strip_prefix("col")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: usize = digits.parse().ok()?;
    if n == 0 { None } else { Some(n - 1) }
}

fn parse_select_item(s: &str) -> Result<SelectItem, Value> {
    let s = s.trim();
    if let Some(col) = parse_col_ref(s) {
        return Ok(SelectItem::Column(col));
    }
    if let Some(open) = s.find('(') {
        if s.ends_with(')') {
            let func_name = &s[..open];
            if let Some(func) = AggFunc::parse(func_name.trim()) {
                let inner = &s[open + 1..s.len() - 1];
                if let Some(col) = parse_col_ref(inner) {
                    return Ok(SelectItem::Agg(func, col));
                }
            }
        }
    }
    Err(parse_error(format!(
        "Unable to parse query string for Function QUERY parameter 2: unsupported SELECT expression '{s}'"
    )))
}

fn parse_select(content: &str, ncols: usize) -> Result<Vec<SelectItem>, Value> {
    let items: Result<Vec<SelectItem>, Value> = split_top_level_commas(content)
        .into_iter()
        .map(parse_select_item)
        .collect();
    let items = items?;
    for item in &items {
        let col = match item {
            SelectItem::Column(c) => *c,
            SelectItem::Agg(_, c) => *c,
        };
        if col >= ncols {
            return Err(parse_error(format!(
                "Unable to parse query string for Function QUERY parameter 2: column Col{} is out of range",
                col + 1
            )));
        }
    }
    Ok(items)
}

fn parse_literal(s: &str) -> Option<Value> {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        return Some(Value::Text(inner.to_string()));
    }
    match s.to_ascii_lowercase().as_str() {
        "true" => return Some(Value::Bool(true)),
        "false" => return Some(Value::Bool(false)),
        _ => {}
    }
    s.parse::<f64>().ok().map(Value::Number)
}

fn parse_condition(s: &str) -> Result<Condition, Value> {
    let trimmed = s.trim();
    let lower = trimmed.to_ascii_lowercase();

    // `ColN is not null` / `ColN is null`
    if let Some(rest) = lower.strip_suffix("is not null") {
        if let Some(col) = parse_col_ref(&trimmed[..rest.len()]) {
            return Ok(Condition { col, op: CondOp::IsNotNull, value: None });
        }
    }
    if let Some(rest) = lower.strip_suffix("is null") {
        if let Some(col) = parse_col_ref(&trimmed[..rest.len()]) {
            return Ok(Condition { col, op: CondOp::IsNull, value: None });
        }
    }

    // `ColN <op> value` — check two-character operators before one-character ones.
    const OPS: [(&str, CondOp); 6] = [
        ("!=", CondOp::Ne),
        ("<>", CondOp::Ne),
        ("<=", CondOp::Le),
        (">=", CondOp::Ge),
        ("<", CondOp::Lt),
        (">", CondOp::Gt),
    ];
    for (token, op) in OPS {
        if let Some(idx) = trimmed.find(token) {
            let col = parse_col_ref(&trimmed[..idx])
                .ok_or_else(|| parse_error(format!("Unable to parse query string for Function QUERY parameter 2: invalid WHERE condition '{s}'")))?;
            let value = parse_literal(&trimmed[idx + token.len()..])
                .ok_or_else(|| parse_error(format!("Unable to parse query string for Function QUERY parameter 2: invalid WHERE value in '{s}'")))?;
            return Ok(Condition { col, op, value: Some(value) });
        }
    }
    if let Some(idx) = trimmed.find('=') {
        let col = parse_col_ref(&trimmed[..idx])
            .ok_or_else(|| parse_error(format!("Unable to parse query string for Function QUERY parameter 2: invalid WHERE condition '{s}'")))?;
        let value = parse_literal(&trimmed[idx + 1..])
            .ok_or_else(|| parse_error(format!("Unable to parse query string for Function QUERY parameter 2: invalid WHERE value in '{s}'")))?;
        return Ok(Condition { col, op: CondOp::Eq, value: Some(value) });
    }

    Err(parse_error(format!(
        "Unable to parse query string for Function QUERY parameter 2: invalid WHERE condition '{s}'"
    )))
}

fn parse_where(content: &str, ncols: usize) -> Result<WhereClause, Value> {
    let and_parts = split_on_word(content, "and");
    let or_parts = split_on_word(content, "or");
    let (parts, join) = if and_parts.len() > 1 && or_parts.len() > 1 {
        return Err(parse_error(
            "Unable to parse query string for Function QUERY parameter 2: mixing AND/OR in WHERE is not supported",
        ));
    } else if and_parts.len() > 1 {
        (and_parts, BoolJoin::And)
    } else if or_parts.len() > 1 {
        (or_parts, BoolJoin::Or)
    } else {
        (and_parts, BoolJoin::And)
    };

    let conditions: Result<Vec<Condition>, Value> = parts.into_iter().map(parse_condition).collect();
    let conditions = conditions?;
    for c in &conditions {
        if c.col >= ncols {
            return Err(parse_error(format!(
                "Unable to parse query string for Function QUERY parameter 2: column Col{} is out of range",
                c.col + 1
            )));
        }
    }
    Ok(WhereClause { conditions, join })
}

fn parse_col_list(content: &str, ncols: usize, clause: &str) -> Result<Vec<usize>, Value> {
    let cols: Result<Vec<usize>, Value> = split_top_level_commas(content)
        .into_iter()
        .map(|s| {
            parse_col_ref(s).ok_or_else(|| {
                parse_error(format!(
                    "Unable to parse query string for Function QUERY parameter 2: invalid column reference '{s}' in {clause}"
                ))
            })
        })
        .collect();
    let cols = cols?;
    for &c in &cols {
        if c >= ncols {
            return Err(parse_error(format!(
                "Unable to parse query string for Function QUERY parameter 2: column Col{} is out of range",
                c + 1
            )));
        }
    }
    Ok(cols)
}

/// Extract the column an ORDER BY item refers to. Accepts a bare `ColN` or
/// an aggregate-function wrapper `func(ColN)` (e.g. `sum(Col3)`) — the
/// function name itself is not retained; ordering by an aggregate is
/// resolved against the matching aggregated SELECT item by column alone
/// (see `exec::sort_grouped_rows`), same as Google Sheets requires the
/// ORDER BY aggregate to already appear in SELECT.
fn parse_order_col_ref(s: &str) -> Option<usize> {
    if let Some(col) = parse_col_ref(s) {
        return Some(col);
    }
    let open = s.find('(')?;
    if !s.ends_with(')') {
        return None;
    }
    parse_col_ref(&s[open + 1..s.len() - 1])
}

fn parse_order_by(content: &str, ncols: usize) -> Result<Vec<OrderItem>, Value> {
    split_top_level_commas(content)
        .into_iter()
        .map(|s| {
            let s = s.trim();
            let lower = s.to_ascii_lowercase();
            let (col_part, dir) = if let Some(rest) = lower.strip_suffix("desc") {
                (&s[..rest.len()], SortDir::Desc)
            } else if let Some(rest) = lower.strip_suffix("asc") {
                (&s[..rest.len()], SortDir::Asc)
            } else {
                (s, SortDir::Asc)
            };
            let col = parse_order_col_ref(col_part.trim()).ok_or_else(|| {
                parse_error(format!(
                    "Unable to parse query string for Function QUERY parameter 2: invalid column reference '{s}' in ORDER BY"
                ))
            })?;
            if col >= ncols {
                return Err(parse_error(format!(
                    "Unable to parse query string for Function QUERY parameter 2: column Col{} is out of range",
                    col + 1
                )));
            }
            Ok(OrderItem { col, dir })
        })
        .collect()
}

fn parse_limit(content: &str) -> Result<usize, Value> {
    content.trim().parse::<usize>().map_err(|_| {
        parse_error(format!(
            "Unable to parse query string for Function QUERY parameter 2: invalid LIMIT value '{}'",
            content.trim()
        ))
    })
}

fn parse_label(content: &str, select: &[SelectItem]) -> Result<Vec<(SelectItem, String)>, Value> {
    split_top_level_commas(content)
        .into_iter()
        .map(|s| {
            let s = s.trim();
            let quote_start = s.find('\'').ok_or_else(|| {
                parse_error(format!(
                    "Unable to parse query string for Function QUERY parameter 2: missing quoted label text in '{s}'"
                ))
            })?;
            let expr = s[..quote_start].trim();
            let rest = &s[quote_start + 1..];
            let quote_end = rest.rfind('\'').ok_or_else(|| {
                parse_error(format!(
                    "Unable to parse query string for Function QUERY parameter 2: unterminated label text in '{s}'"
                ))
            })?;
            let label_text = rest[..quote_end].to_string();
            let item = parse_select_item(expr)?;
            if !select.contains(&item) {
                return Err(parse_error(format!(
                    "Unable to parse query string for Function QUERY parameter 2: LABEL target '{expr}' is not in the SELECT list"
                )));
            }
            Ok((item, label_text))
        })
        .collect()
}

/// Parse the full QUERY language string against a data set with `ncols`
/// columns. `ncols` is used to validate every column reference up front.
pub fn parse(query: &str, ncols: usize) -> Result<ParsedQuery, Value> {
    let clauses = split_clauses(query)?;

    let mut select: Option<Vec<SelectItem>> = None;
    let mut where_clause: Option<WhereClause> = None;
    let mut group_by: Vec<usize> = Vec::new();
    let mut order_by: Vec<OrderItem> = Vec::new();
    let mut limit: Option<usize> = None;
    let mut label_content: Option<&str> = None;

    for (kw, content) in &clauses {
        match *kw {
            "select" => select = Some(parse_select(content, ncols)?),
            "where" => where_clause = Some(parse_where(content, ncols)?),
            "group by" => group_by = parse_col_list(content, ncols, "GROUP BY")?,
            "order by" => order_by = parse_order_by(content, ncols)?,
            "limit" => limit = Some(parse_limit(content)?),
            "label" => label_content = Some(content),
            _ => unreachable!(),
        }
    }

    let select = select.unwrap_or_else(|| (0..ncols).map(SelectItem::Column).collect());

    // Every bare column in SELECT must be a GROUP BY key when GROUP BY is used.
    if !group_by.is_empty() {
        for item in &select {
            if let SelectItem::Column(c) = item {
                if !group_by.contains(c) {
                    return Err(parse_error(format!(
                        "Unable to parse query string for Function QUERY parameter 2: Col{} must appear in the GROUP BY clause or be aggregated",
                        c + 1
                    )));
                }
            }
        }
    } else {
        let has_agg = select.iter().any(|s| matches!(s, SelectItem::Agg(..)));
        let has_bare = select.iter().any(|s| matches!(s, SelectItem::Column(_)));
        if has_agg && has_bare {
            return Err(parse_error(
                "Unable to parse query string for Function QUERY parameter 2: cannot mix aggregated and non-aggregated columns without GROUP BY",
            ));
        }
    }

    let labels = match label_content {
        Some(content) => parse_label(content, &select)?,
        None => Vec::new(),
    };

    Ok(ParsedQuery { select, where_clause, group_by, order_by, limit, labels })
}
