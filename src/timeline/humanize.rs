use chrono::{DateTime, Utc};
use std::time::Duration;

pub fn humanize_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs == 3600 {
        return "1 hr ago".to_string();
    }
    if secs == 86400 {
        return "1 day ago".to_string();
    }
    match secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=10799 => "1-3 hrs ago".to_string(),
        10800..=21599 => "3-6 hrs ago".to_string(),
        21600..=43199 => "6-12 hrs ago".to_string(),
        43200..=86399 => "12-24 hrs ago".to_string(),
        86400..=259199 => "2-3 days ago".to_string(),
        259200..=604799 => "3-7 days ago".to_string(),
        604800..=1209599 => "1-2 weeks ago".to_string(),
        1209600..=2419199 => "2-4 weeks ago".to_string(),
        _ => "1+ months ago".to_string(),
    }
}

pub fn humanize_time_range(earliest: DateTime<Utc>, latest: DateTime<Utc>) -> String {
    let now = Utc::now();
    let earliest_dur = (now - earliest).to_std().unwrap_or(Duration::ZERO);
    let latest_dur = (now - latest).to_std().unwrap_or(Duration::ZERO);
    let earliest_str = humanize_duration(earliest_dur);
    let latest_str = humanize_duration(latest_dur);
    if earliest_str == latest_str {
        earliest_str
    } else {
        format!("{earliest_str} to {latest_str}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_duration_zero_seconds() {
        assert_eq!(humanize_duration(Duration::from_secs(0)), "just now");
    }

    #[test]
    fn test_humanize_duration_30_seconds() {
        assert_eq!(humanize_duration(Duration::from_secs(30)), "just now");
    }

    #[test]
    fn test_humanize_duration_59_seconds() {
        assert_eq!(humanize_duration(Duration::from_secs(59)), "just now");
    }

    #[test]
    fn test_humanize_duration_exactly_60_seconds() {
        assert_eq!(humanize_duration(Duration::from_secs(60)), "1m ago");
    }

    #[test]
    fn test_humanize_duration_5_minutes() {
        assert_eq!(humanize_duration(Duration::from_secs(300)), "5m ago");
    }

    #[test]
    fn test_humanize_duration_59_minutes() {
        assert_eq!(humanize_duration(Duration::from_secs(3540)), "59m ago");
    }

    #[test]
    fn test_humanize_duration_exactly_3600_seconds() {
        assert_eq!(humanize_duration(Duration::from_secs(3600)), "1 hr ago");
    }

    #[test]
    fn test_humanize_duration_2_hours() {
        assert_eq!(humanize_duration(Duration::from_secs(7200)), "1-3 hrs ago");
    }

    #[test]
    fn test_humanize_duration_4_hours() {
        assert_eq!(humanize_duration(Duration::from_secs(14400)), "3-6 hrs ago");
    }

    #[test]
    fn test_humanize_duration_8_hours() {
        assert_eq!(humanize_duration(Duration::from_secs(28800)), "6-12 hrs ago");
    }

    #[test]
    fn test_humanize_duration_18_hours() {
        assert_eq!(humanize_duration(Duration::from_secs(64800)), "12-24 hrs ago");
    }

    #[test]
    fn test_humanize_duration_exactly_86400_seconds() {
        assert_eq!(humanize_duration(Duration::from_secs(86400)), "1 day ago");
    }

    #[test]
    fn test_humanize_duration_2_days() {
        assert_eq!(humanize_duration(Duration::from_secs(172800)), "2-3 days ago");
    }

    #[test]
    fn test_humanize_duration_5_days() {
        assert_eq!(humanize_duration(Duration::from_secs(432000)), "3-7 days ago");
    }

    #[test]
    fn test_humanize_duration_1_week() {
        assert_eq!(humanize_duration(Duration::from_secs(604800)), "1-2 weeks ago");
    }

    #[test]
    fn test_humanize_duration_2_weeks() {
        assert_eq!(humanize_duration(Duration::from_secs(1209600)), "2-4 weeks ago");
    }

    #[test]
    fn test_humanize_duration_3_weeks() {
        assert_eq!(humanize_duration(Duration::from_secs(1814400)), "2-4 weeks ago");
    }

    #[test]
    fn test_humanize_duration_5_weeks() {
        assert_eq!(humanize_duration(Duration::from_secs(3024000)), "1+ months ago");
    }

    #[test]
    fn test_humanize_time_range_same_bucket() {
        let now = Utc::now();
        let t1 = now - chrono::Duration::seconds(5);
        let t2 = now - chrono::Duration::seconds(3);
        let result = humanize_time_range(t1, t2);
        assert_eq!(result, "just now");
    }

    #[test]
    fn test_humanize_time_range_different_buckets() {
        let now = Utc::now();
        let t1 = now - chrono::Duration::hours(5);
        let t2 = now - chrono::Duration::minutes(2);
        let result = humanize_time_range(t1, t2);
        assert!(result.contains(" to "));
    }

    #[test]
    fn test_humanize_time_range_same_time() {
        let now = Utc::now();
        let t = now - chrono::Duration::hours(1);
        let result = humanize_time_range(t, t);
        assert!(!result.contains(" to "));
    }
}
