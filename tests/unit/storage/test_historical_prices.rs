use chrono::{DateTime, NaiveDateTime, Utc};
use ig_client::storage::historical_prices::{parse_snapshot_time, StorageStats};

fn dt_utc(s: &str, fmt: &str) -> DateTime<Utc> {
    let ndt = NaiveDateTime::parse_from_str(s, fmt).unwrap();
    DateTime::from_naive_utc_and_offset(ndt, Utc)
}

#[test]
fn parse_snapshot_time_accepts_multiple_formats() {
    // With slashes and seconds
    let d1 = parse_snapshot_time("2025/10/20 19:22:33").unwrap();
    assert_eq!(d1, dt_utc("2025-10-20 19:22:33", "%Y-%m-%d %H:%M:%S"));

    // With dashes and seconds
    let d2 = parse_snapshot_time("2025-10-20 19:22:33").unwrap();
    assert_eq!(d2, dt_utc("2025-10-20 19:22:33", "%Y-%m-%d %H:%M:%S"));

    // With slashes, no seconds
    let d3 = parse_snapshot_time("2025/10/20 19:22").unwrap();
    assert_eq!(d3, dt_utc("2025-10-20 19:22:00", "%Y-%m-%d %H:%M:%S"));

    // With dashes, no seconds
    let d4 = parse_snapshot_time("2025-10-20 19:22").unwrap();
    assert_eq!(d4, dt_utc("2025-10-20 19:22:00", "%Y-%m-%d %H:%M:%S"));
}

#[test]
fn parse_snapshot_time_rejects_invalid_inputs() {
    for bad in [
        "",
        "2025/13/01 00:00:00", // invalid month
        "2025-10-32 00:00:00", // invalid day
        "2025-10-20T19:22:33Z", // unsupported separator/timezone
        "20-10-2025 00:00:00",   // wrong order
        "2025-10-20",
    ] {
        assert!(parse_snapshot_time(bad).is_err(), "should fail for {bad}");
    }
}

#[test]
fn storage_stats_default_is_zero() {
    let s = StorageStats::default();
    assert_eq!(s.inserted, 0);
    assert_eq!(s.updated, 0);
    assert_eq!(s.skipped, 0);
    assert_eq!(s.total_processed, 0);
}
