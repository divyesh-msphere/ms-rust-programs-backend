// domain/store_query.rs
//
// The `search` and `filter` strings of a program's stores list, in the same grammar as the programs list.
//
//   search=abc                               the search box: matches the BU location code, the website or the customer name
//   search=bu_location=abc                   only the BU location code
//   filter=bu_location.starts_with=BU&customer_name.is_empty&website=[w1,w2]&status!=ended
//
// `bu_location` (its code), `customer_name` and `website` (its code, name or domain) take an operator
// (`field.operator=value`, default contains). `website` and `status` without an operator take one value or
// a `[a,b]` list that matches exactly (`key!=value`, or `.not_equals`, keeps everything except those).
// Terms are separated by `&` (an `&` inside `[...]` or quotes does not split). All terms are ANDed; the values
// of one list are ORed. An empty value switches that term off.
use crate::application::errors::application_error::ApplicationError;
use crate::domain::enrollment::STATUSES;
use crate::domain::program_query::{TextOp, bad, split_terms, unquote, values};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreField {
    /// The BU location code.
    BuLocation,
    /// The website code, name or domain.
    Website,
    CustomerName,
    /// Every field above (a bare `search=abc`).
    Any,
}

impl StoreField {
    pub const ALL: &'static str = "bu_location, website, customer_name";

    fn parse(name: &str) -> Result<Self, ApplicationError> {
        match name {
            "bu_location" => Ok(Self::BuLocation),
            "website" => Ok(Self::Website),
            "customer_name" => Ok(Self::CustomerName),
            other => Err(bad(&format!("field {other:?} is not known; use one of: {}", Self::ALL))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreTerm {
    pub field: StoreField,
    pub op: TextOp,
    /// Empty for `is_empty` / `is_not_empty`.
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoreFilter {
    /// Text conditions with their operators.
    pub text: Vec<StoreTerm>,
    /// Statuses to keep / to leave out.
    pub statuses: Vec<String>,
    pub not_statuses: Vec<String>,
    /// Website codes, names or domains (exact, any of) to keep / to leave out.
    pub websites: Vec<String>,
    pub not_websites: Vec<String>,
}

/// One text term, `field[.operator][=value]`. `None` when the value is empty (a cleared box).
fn parse_text_term(term: &str) -> Result<Option<StoreTerm>, ApplicationError> {
    let (left, right) = match term.split_once('=') {
        Some((l, r)) => (l.trim(), Some(unquote(r))),
        None => (term.trim(), None),
    };
    let (field, op) = match left.split_once('.') {
        Some((f, o)) => (StoreField::parse(&f.trim().to_lowercase())?, TextOp::parse(&o.trim().to_lowercase())?),
        None => (StoreField::parse(&left.to_lowercase())?, TextOp::Contains),
    };
    if !op.takes_value() {
        if right.as_deref().is_some_and(|v| !v.is_empty()) {
            return Err(bad(&format!("{term:?}: is_empty and is_not_empty take no value")));
        }
        if field != StoreField::CustomerName {
            return Err(bad("only customer_name can be empty, so is_empty / is_not_empty only apply to it"));
        }
        return Ok(Some(StoreTerm { field, op, value: String::new() }));
    }
    match right {
        None => Err(bad(&format!("{term:?}: expected field=value, e.g. bu_location=BU-1"))),
        Some(v) if v.is_empty() => Ok(None),
        Some(value) => Ok(Some(StoreTerm { field, op, value })),
    }
}

/// The search box: bare text matches every field; `field=text` one field. Always a contains match.
pub fn parse_search(input: Option<&str>) -> Result<Vec<StoreTerm>, ApplicationError> {
    let mut out = Vec::new();
    for term in split_terms(input.unwrap_or("")) {
        let left = term.split('=').next().unwrap_or("").trim().to_lowercase();
        let key = left.split('.').next().unwrap_or("").to_string();
        let is_field = term.contains('=') && StoreField::parse(&key).is_ok();
        if is_field {
            if term.split_once('=').is_some_and(|(l, _)| l.contains('.')) {
                return Err(bad("search is always a contains match; use filter for operators"));
            }
            out.extend(parse_text_term(&term)?);
        } else {
            let text = unquote(&term);
            if !text.is_empty() {
                out.push(StoreTerm { field: StoreField::Any, op: TextOp::Contains, value: text });
            }
        }
    }
    Ok(out)
}

pub fn parse_filter(input: Option<&str>) -> Result<StoreFilter, ApplicationError> {
    let mut filter = StoreFilter::default();
    for term in split_terms(input.unwrap_or("")) {
        let (left, raw) = match term.split_once('=') {
            Some((l, r)) => (l.trim().to_lowercase(), Some(r)),
            None => (term.trim().to_lowercase(), None),
        };
        let (left, mut negated) = match left.strip_suffix('!') {
            Some(k) => (k.trim_end().to_string(), true),
            None => (left, false),
        };
        let (key, op) = match left.split_once('.') {
            Some((k, o)) => (k.trim().to_string(), Some(o.trim().to_string())),
            None => (left.clone(), None),
        };
        match key.as_str() {
            "status" | "state" | "website" => {
                // `.equals` / `.in` keep the plain list meaning, `.not_equals` flips it.
                match op.as_deref() {
                    None | Some("equals" | "equal" | "eq" | "is" | "in") => {}
                    Some("not_equals" | "not_equal" | "not_eq" | "ne" | "not" | "is_not" | "not_in") => negated = !negated,
                    Some(o) if key == "website" => {
                        filter.text.extend(parse_text_term(&format!("website.{o}={}", raw.unwrap_or("")))?);
                        continue;
                    }
                    Some(o) => {
                        return Err(bad(&format!("{term:?}: operator {o:?} is not known for {key}; use equals or not_equals (or `!=`)")));
                    }
                }
                let Some(raw) = raw else {
                    return Err(bad(&format!("{term:?}: expected key=value or key=[a,b]")));
                };
                let list = values(raw);
                if key == "website" {
                    if negated { filter.not_websites.extend(list) } else { filter.websites.extend(list) }
                } else {
                    // `status=active,suspended` (comma separated) is accepted too.
                    for v in list.iter().flat_map(|v| v.split(',')).map(str::trim).filter(|v| !v.is_empty()) {
                        let v = v.to_lowercase();
                        if !STATUSES.contains(&v.as_str()) {
                            return Err(bad(&format!("status {v:?} must be one of: {}", STATUSES.join(", "))));
                        }
                        if negated { filter.not_statuses.push(v) } else { filter.statuses.push(v) }
                    }
                }
            }
            "bu_location" | "customer_name" => {
                if negated {
                    return Err(bad("use bu_location.not_equals=... (or not_contains) instead of !="));
                }
                filter.text.extend(parse_text_term(&term)?);
            }
            other => {
                return Err(bad(&format!(
                    "filter key {other:?} is not known; use bu_location, website, customer_name or status"
                )));
            }
        }
    }
    Ok(filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_bare_text_matches_every_field() {
        let t = parse_search(Some("acme")).unwrap();
        assert_eq!(t, vec![StoreTerm { field: StoreField::Any, op: TextOp::Contains, value: "acme".into() }]);
        let t = parse_search(Some("bu_location=BU-1")).unwrap();
        assert_eq!(t[0].field, StoreField::BuLocation);
        assert!(parse_search(Some("bu_location.equals=x")).is_err());
        assert!(parse_search(Some("")).unwrap().is_empty());
    }

    #[test]
    fn filter_text_and_lists() {
        let f = parse_filter(Some("bu_location.starts_with=BU&customer_name.is_empty&website=[w1, w2]&status!=ended")).unwrap();
        assert_eq!(f.text.len(), 2);
        assert_eq!(f.text[0].op, TextOp::StartsWith);
        assert_eq!(f.text[1].op, TextOp::IsEmpty);
        assert_eq!(f.websites, vec!["w1", "w2"]);
        assert_eq!(f.not_statuses, vec!["ended"]);
    }

    #[test]
    fn filter_operator_forms_and_errors() {
        let f = parse_filter(Some("status.not_equal=active&website.not_equals=w1&website.contains=main&status.in=[pending,ended]")).unwrap();
        assert_eq!(f.not_statuses, vec!["active"]);
        assert_eq!(f.not_websites, vec!["w1"]);
        assert_eq!(f.text[0].field, StoreField::Website);
        assert_eq!(f.statuses, vec!["pending", "ended"]);
        assert_eq!(parse_filter(Some("status=pending,active")).unwrap().statuses, vec!["pending", "active"]);
        assert!(parse_filter(Some("status=gone")).is_err());
        assert!(parse_filter(Some("colour=red")).is_err());
        assert!(parse_filter(Some("bu_location.is_empty")).is_err());
        assert!(parse_filter(Some("status.contains=x")).is_err());
        assert_eq!(parse_filter(Some("website=&status=[]&bu_location=")).unwrap(), StoreFilter::default());
    }
}
