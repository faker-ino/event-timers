use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn calculate_tyria_time(utc_timestamp: i64) -> (i32, i32) {
    let reference_time: i64 = 1759264200; // 2025-09-30 17:30:00 UTC-3 = Tyrian 06:00

    // Work in seconds for precision, then convert to Tyrian minutes
    let real_seconds_elapsed = utc_timestamp - reference_time;

    // 1 real second = 12 Tyrian minutes / 60 seconds = 0.2 Tyrian minutes = 12 Tyrian seconds
    // So: 1 real second = 12 Tyrian seconds
    let tyria_seconds_elapsed = real_seconds_elapsed * 12;

    // Convert to Tyrian minutes
    let tyria_minutes_elapsed = tyria_seconds_elapsed / 60;

    // Start at 6:00 (360 minutes into the day)
    let total_tyria_minutes = 360 + tyria_minutes_elapsed;

    // Wrap around 24-hour cycle (1440 minutes)
    let tyria_minutes_in_day = total_tyria_minutes.rem_euclid(1440);

    let hours = (tyria_minutes_in_day / 60) as i32;
    let minutes = (tyria_minutes_in_day % 60) as i32;

    (hours, minutes)
}

pub fn format_time_only(timestamp: i64) -> String {
    use chrono::{DateTime, Duration, Utc};

    let Some(datetime) = DateTime::from_timestamp(timestamp, 0) else {
        return "--:--".to_string();
    };

    let offset_seconds = local_display_offset_seconds(timestamp);
    (datetime.with_timezone(&Utc) + Duration::seconds(offset_seconds))
        .format("%H:%M")
        .to_string()
}

fn local_display_offset_seconds(timestamp: i64) -> i64 {
    if let Ok(tz) = std::env::var("TZ") {
        if let Some(offset) = parse_posix_tz_offset_seconds(&tz, timestamp) {
            return offset;
        }
    }

    0
}

fn parse_posix_tz_offset_seconds(tz: &str, timestamp: i64) -> Option<i64> {
    let bytes = tz.as_bytes();
    let mut idx = 0;

    while bytes.get(idx).is_some_and(|b| b.is_ascii_alphabetic()) {
        idx += 1;
    }

    if idx == 0 || idx >= bytes.len() {
        return None;
    }

    let (standard_offset, next_idx) = parse_posix_offset_seconds(&tz[idx..])?;
    idx += next_idx;

    let has_dst = bytes.get(idx).is_some_and(|b| b.is_ascii_alphabetic());
    if !has_dst || !is_us_dst(timestamp, standard_offset) {
        return Some(standard_offset);
    }

    Some(standard_offset + 60 * 60)
}

fn parse_posix_offset_seconds(input: &str) -> Option<(i64, usize)> {
    let bytes = input.as_bytes();
    let mut idx = 0;
    let mut sign = -1;

    if let Some(b) = bytes.get(idx) {
        if *b == b'+' {
            idx += 1;
        } else if *b == b'-' {
            sign = 1;
            idx += 1;
        }
    }

    let start = idx;
    while bytes.get(idx).is_some_and(|b| b.is_ascii_digit()) {
        idx += 1;
    }

    if idx == start {
        return None;
    }

    let hours: i64 = input[start..idx].parse().ok()?;
    Some((sign * hours * 60 * 60, idx))
}

fn is_us_dst(timestamp: i64, standard_offset_seconds: i64) -> bool {
    use chrono::{DateTime, Datelike, NaiveTime, Weekday};

    let Some(utc) = DateTime::from_timestamp(timestamp, 0) else {
        return false;
    };
    let year = utc.year();

    let dst_start_local = nth_weekday_of_month(year, 3, Weekday::Sun, 2)
        .and_time(NaiveTime::from_hms_opt(2, 0, 0).unwrap());
    let dst_end_local = nth_weekday_of_month(year, 11, Weekday::Sun, 1)
        .and_time(NaiveTime::from_hms_opt(2, 0, 0).unwrap());

    let dst_start_utc = local_standard_to_utc(dst_start_local, standard_offset_seconds);
    let dst_end_utc = local_standard_to_utc(dst_end_local, standard_offset_seconds + 60 * 60);

    timestamp >= dst_start_utc && timestamp < dst_end_utc
}

fn nth_weekday_of_month(
    year: i32,
    month: u32,
    weekday: chrono::Weekday,
    nth: u32,
) -> chrono::NaiveDate {
    use chrono::{Datelike, Duration, NaiveDate};

    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let days_to_weekday = (7 + weekday.num_days_from_monday() as i64
        - first.weekday().num_days_from_monday() as i64)
        % 7;
    first + Duration::days(days_to_weekday + 7 * (nth as i64 - 1))
}

fn local_standard_to_utc(local: chrono::NaiveDateTime, offset_seconds: i64) -> i64 {
    local.and_utc().timestamp() - offset_seconds
}
