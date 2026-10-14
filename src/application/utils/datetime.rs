use chrono::{DateTime, Utc};

/// Formats a UTC timestamp as `YYYY-MM-DDTHH:MM:SS[.ffffff]` — date and time
/// only, no timezone offset or `Z` marker. The value itself is still always
/// UTC (never converted to any other timezone); only the trailing offset is
/// omitted from the string.
pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S%.f").to_string()
}
