//! Shared civil-date helpers for orchestration and audit.
//!
//! The bash CLI shells out to `date`, so we do too for exact parity (local
//! timezone for ages, UTC for the completion stamp), falling back to a pure
//! computation from `SystemTime` when `date` is unavailable.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Today as `YYYY-MM-DD` in UTC — mirrors `orch_today` (`date -u +%Y-%m-%d`),
/// the stamp written by `aps complete`.
pub fn today_utc_ymd() -> String {
    Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| s.len() == 10)
        .unwrap_or_else(|| {
            let days = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| (d.as_secs() / 86_400) as i64)
                .unwrap_or(0);
            let (y, m, d) = civil_from_days(days);
            format!("{y:04}-{m:02}-{d:02}")
        })
}

/// A `YYYYMMDD-HHMMSS` timestamp for backup directory names — mirrors
/// `date +%Y%m%d-%H%M%S` in scaffold/upgrade. Falls back to epoch seconds
/// when `date` is unavailable.
pub fn now_stamp() -> String {
    Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| s.len() == 15)
        .unwrap_or_else(|| {
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("epoch-{secs}")
        })
}

/// Today as civil days since the epoch, in the *local* timezone — bash's
/// `date +%s` anchors at local midnight, so ages computed against it use the
/// local civil day. std has no timezone access, so ask `date` like bash does;
/// fall back to UTC when unavailable.
pub fn today_civil_days() -> i64 {
    Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| parse_civil_date(String::from_utf8_lossy(&out.stdout).trim()))
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| (d.as_secs() / 86_400) as i64)
                .unwrap_or(0)
        })
}

/// Days since the Unix epoch for a `YYYY-MM-DD` date (Howard Hinnant's
/// civil-date algorithm). Returns None for malformed dates.
pub fn parse_civil_date(date: &str) -> Option<i64> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let y: i64 = date[..4].parse().ok()?;
    let m: i64 = date[5..7].parse().ok()?;
    let d: i64 = date[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// Inverse of `parse_civil_date`: civil `(year, month, day)` from days since
/// the Unix epoch (Hinnant's `civil_from_days`). Used only for the fallback
/// when shelling out to `date` fails.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_round_trips() {
        for date in ["1970-01-01", "2026-06-30", "2000-02-29", "1999-12-31"] {
            let days = parse_civil_date(date).unwrap();
            let (y, m, d) = civil_from_days(days);
            assert_eq!(format!("{y:04}-{m:02}-{d:02}"), date);
        }
    }

    #[test]
    fn epoch_is_day_zero() {
        assert_eq!(parse_civil_date("1970-01-01"), Some(0));
        assert_eq!(parse_civil_date("1970-01-02"), Some(1));
    }

    #[test]
    fn rejects_malformed() {
        assert_eq!(parse_civil_date("2026-13-01"), None);
        assert_eq!(parse_civil_date("not-a-date"), None);
        assert_eq!(parse_civil_date("2026-06-30 "), None);
    }
}
