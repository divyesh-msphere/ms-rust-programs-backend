// domain/program_query.rs
//
// The `search` and `filter` strings of the programs list.
//
//   search=program=abc                       the search box: the name or slug contains "abc"
//   filter=program.not_contains=abc&description.is_not_empty&domain=[w1,TechyParts]&status=active
//
// `search` takes only `program=<text>` (a contains match). Everything with an operator lives in
// `filter`, next to the value lists: `program` / `description` take an operator
// (`field.operator=value`, default contains) and `domain`, `status`, `level` take a value or a
// `[a,b]` list (`key!=value` keeps everything except those). Terms are separated by `&` (an `&` inside `[...]` or quotes does not split; commas only separate the
// items of a `[a,b]` list). Over HTTP the terms may also be sent as plain query parameters, see the controller. All
// terms are ANDed; the values of one list are ORed. An empty value (`domain=`, `domain=[]`,
// `program=`) switches that term off, so a cleared box needs no special case.
use crate::application::errors::application_error::ApplicationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchField {
    /// The name or the slug.
    Program,
    Description,
}

impl SearchField {
    pub const ALL: &'static str = "program, description";

    fn parse(name: &str) -> Result<Self, ApplicationError> {
        match name {
            "program" => Ok(Self::Program),
            "description" => Ok(Self::Description),
            other => Err(bad(&format!("search field {other:?} is not known; use one of: {}", Self::ALL))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOp {
    Contains,
    NotContains,
    Equals,
    NotEquals,
    StartsWith,
    EndsWith,
    IsEmpty,
    IsNotEmpty,
}

impl TextOp {
    pub const ALL: &'static str =
        "contains, not_contains, equals, not_equals, starts_with, ends_with, is_empty, is_not_empty";

    fn parse(name: &str) -> Result<Self, ApplicationError> {
        match name {
            "contains" => Ok(Self::Contains),
            "not_contains" => Ok(Self::NotContains),
            "equals" => Ok(Self::Equals),
            "not_equals" => Ok(Self::NotEquals),
            "starts_with" => Ok(Self::StartsWith),
            "ends_with" => Ok(Self::EndsWith),
            "is_empty" => Ok(Self::IsEmpty),
            "is_not_empty" => Ok(Self::IsNotEmpty),
            other => Err(bad(&format!("operator {other:?} is not known; use one of: {}", Self::ALL))),
        }
    }

    fn takes_value(self) -> bool {
        !matches!(self, Self::IsEmpty | Self::IsNotEmpty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTerm {
    pub field: SearchField,
    pub op: TextOp,
    /// Empty for `is_empty` / `is_not_empty`.
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListFilter {
    /// `program` / `description` conditions with their operators.
    pub text: Vec<SearchTerm>,
    /// Website ids or names, any of.
    pub domains: Vec<String>,
    /// `active` flags to keep: `status=active` is `[true]`, `status=inactive` is `[false]`.
    pub active: Vec<bool>,
    pub levels: Vec<String>,
    /// The `!=` forms: `domain!=w1`, `status!=active`, `level!=user` keep what is NOT in the list.
    pub not_domains: Vec<String>,
    pub not_active: Vec<bool>,
    pub not_levels: Vec<String>,
}

fn bad(msg: &str) -> ApplicationError {
    ApplicationError::validation_error(msg)
}

/// Splits on `&` that is outside `[...]` and quotes.
fn split_terms(input: &str) -> Vec<String> {
    split_on(input, '&')
}

/// The items of a `[a,b]` list.
fn split_items(input: &str) -> Vec<String> {
    split_on(input, ',')
}

fn split_on(input: &str, separator: char) -> Vec<String> {
    let (mut terms, mut current) = (Vec::new(), String::new());
    let (mut depth, mut quote) = (0usize, None::<char>);
    for c in input.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '[') => depth += 1,
            (None, ']') => depth = depth.saturating_sub(1),
            (None, c) if c == separator && depth == 0 => {
                terms.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    terms.push(current);
    terms.into_iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
}

fn unquote(value: &str) -> String {
    let v = value.trim();
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return v[1..v.len() - 1].trim().to_string();
        }
    }
    v.to_string()
}

/// One text term, `field[.operator][=value]`. `None` when the value is empty (a cleared box).
fn parse_text_term(term: &str) -> Result<Option<SearchTerm>, ApplicationError> {
    let (left, right) = match term.split_once('=') {
        Some((l, r)) => (l.trim(), Some(unquote(r))),
        None => (term.trim(), None),
    };
    let (field, op) = match left.split_once('.') {
        Some((f, o)) => (SearchField::parse(&f.trim().to_lowercase())?, TextOp::parse(&o.trim().to_lowercase())?),
        None => (SearchField::parse(&left.to_lowercase())?, TextOp::Contains),
    };
    if !op.takes_value() {
        if right.as_deref().is_some_and(|v| !v.is_empty()) {
            return Err(bad(&format!("{term:?}: is_empty and is_not_empty take no value")));
        }
        if field == SearchField::Program {
            return Err(bad("a program always has a name, so is_empty / is_not_empty only apply to description"));
        }
        return Ok(Some(SearchTerm { field, op, value: String::new() }));
    }
    match right {
        None => Err(bad(&format!("{term:?}: expected field=value, e.g. program=abc"))),
        Some(v) if v.is_empty() => Ok(None),
        Some(value) => Ok(Some(SearchTerm { field, op, value })),
    }
}

/// The search box: `program=abc`. Only a contains match on the name or slug; operators
/// (not_contains, equals, ...) belong to `filter`.
pub fn parse_search(input: Option<&str>) -> Result<Vec<SearchTerm>, ApplicationError> {
    let mut out = Vec::new();
    for term in split_terms(input.unwrap_or("")) {
        let key = term.split('=').next().unwrap_or("").trim().to_lowercase();
        if key != "program" || !term.contains('=') {
            return Err(bad(&format!(
                "search takes program=<text> (a contains match on the name or slug); got {term:?}. \
                 Use filter for operators such as program.not_contains=abc"
            )));
        }
        out.extend(parse_text_term(&term)?);
    }
    Ok(out)
}

fn values(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    let items = match raw.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        Some(inner) => split_items(inner).iter().map(|v| unquote(v)).collect(),
        None => vec![unquote(raw)],
    };
    items.into_iter().filter(|v| !v.is_empty()).collect()
}

pub fn parse_filter(input: Option<&str>) -> Result<ListFilter, ApplicationError> {
    let mut filter = ListFilter::default();
    for term in split_terms(input.unwrap_or("")) {
        let head = term.split(['=', '.']).next().unwrap_or("").trim().to_lowercase();
        if head == "program" || head == "description" {
            filter.text.extend(parse_text_term(&term)?);
            continue;
        }
        let Some((key, raw)) = term.split_once('=') else {
            return Err(bad(&format!(
                "{term:?}: expected key=value or key=[a,b]; keys: program, description, domain, state, level"
            )));
        };
        let list = values(raw);
        let key = key.trim().to_lowercase();
        let (key, negated) = match key.strip_suffix('!') {
            Some(k) => (k.trim_end().to_string(), true),
            None => (key, false),
        };
        match key.as_str() {
            "domain" => if negated { filter.not_domains.extend(list) } else { filter.domains.extend(list) },
            "status" | "state" => {
                for v in list {
                    let flag = match v.to_lowercase().as_str() {
                        "active" => true,
                        "inactive" => false,
                        _ => return Err(bad(&format!("status {v:?} is not known; use active or inactive"))),
                    };
                    if negated { filter.not_active.push(flag) } else { filter.active.push(flag) }
                }
            }
            "level" => {
                for v in list {
                    let v = v.to_lowercase();
                    if !["user", "organization", "location"].contains(&v.as_str()) {
                        return Err(bad(&format!("level {v:?} is not known; use user, organization or location")));
                    }
                    if negated { filter.not_levels.push(v) } else { filter.levels.push(v) }
                }
            }
            other => return Err(bad(&format!("filter key {other:?} is not known; use program, description, domain, state (or status) or level"))),
        }
    }
    Ok(filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_is_a_contains_on_program_and_accepts_quotes() {
        let t = parse_search(Some(r#"program="abc""#)).unwrap();
        assert_eq!(t, vec![SearchTerm { field: SearchField::Program, op: TextOp::Contains, value: "abc".into() }]);
        assert!(parse_search(Some("program=")).unwrap().is_empty());
        assert!(parse_search(None).unwrap().is_empty());
    }

    #[test]
    fn search_rejects_operators_and_other_fields() {
        for bad in ["program.not_contains=x", "description=x", "title=x", "program", "abc"] {
            let err = parse_search(Some(bad)).unwrap_err().to_string();
            assert!(err.contains("program=<text>"), "{bad}: {err}");
        }
    }

    #[test]
    fn filter_carries_the_text_operators() {
        let f = parse_filter(Some("program.starts_with=ab&description.is_not_empty&status=active")).unwrap();
        assert_eq!(f.text[0], SearchTerm { field: SearchField::Program, op: TextOp::StartsWith, value: "ab".into() });
        assert_eq!(f.text[1], SearchTerm { field: SearchField::Description, op: TextOp::IsNotEmpty, value: String::new() });
        assert_eq!(f.active, vec![true]);
        assert_eq!(parse_filter(Some("program=ab")).unwrap().text[0].op, TextOp::Contains);
    }

    #[test]
    fn a_comma_outside_a_list_is_part_of_the_value() {
        let f = parse_filter(Some("program.equals=a,b&status=active")).unwrap();
        assert_eq!(f.text[0].value, "a,b");
        assert_eq!(f.active, vec![true]);
    }

    #[test]
    fn quoted_value_may_hold_a_comma() {
        let f = parse_filter(Some(r#"program.equals="a & b""#)).unwrap();
        assert_eq!(f.text[0].value, "a & b");
    }

    #[test]
    fn text_errors_name_the_valid_choices() {
        assert!(parse_filter(Some("program.like=x")).unwrap_err().to_string().contains("not_contains"));
        assert!(parse_filter(Some("program.is_empty")).is_err());
        assert!(parse_filter(Some("description.is_empty=x")).is_err());
        assert!(parse_filter(Some("program.equals")).is_err());
    }

    #[test]
    fn filter_lists_and_singles() {
        let f = parse_filter(Some("domain=[w1, TechyParts]&status=active&level=[organization,location]")).unwrap();
        assert_eq!(f.domains, vec!["w1", "TechyParts"]);
        assert_eq!(f.active, vec![true]);
        assert_eq!(f.levels, vec!["organization", "location"]);
        let g = parse_filter(Some("domain=w1&status=[active,inactive]")).unwrap();
        assert_eq!(g.domains, vec!["w1"]);
        assert_eq!(g.active, vec![true, false]);
    }

    #[test]
    fn not_equal_forms_exclude() {
        let f = parse_filter(Some("status!=active&domain!=[w1,w2]&level!=user")).unwrap();
        assert_eq!(f.not_active, vec![true]);
        assert_eq!(f.not_domains, vec!["w1", "w2"]);
        assert_eq!(f.not_levels, vec!["user"]);
        assert!(f.active.is_empty() && f.domains.is_empty() && f.levels.is_empty());
        assert!(parse_filter(Some("status!=maybe")).is_err());
    }

    #[test]
    fn empty_filter_values_switch_the_key_off() {
        let f = parse_filter(Some(r#"domain=""&status=[]&level="#)).unwrap();
        assert_eq!(f, ListFilter::default());
    }

    #[test]
    fn filter_errors() {
        assert!(parse_filter(Some("status=maybe")).is_err());
        assert!(parse_filter(Some("level=galaxy")).is_err());
        assert!(parse_filter(Some("colour=red")).is_err());
        assert!(parse_filter(Some("domain")).is_err());
    }
}
