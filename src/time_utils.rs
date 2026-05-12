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
    if timestamp < libc::time_t::MIN as i64 || timestamp > libc::time_t::MAX as i64 {
        return "--:--".to_string();
    }

    let ts: libc::time_t = timestamp as libc::time_t;
    let mut out = std::mem::MaybeUninit::<libc::tm>::uninit();

    #[cfg(unix)]
    let ok = unsafe { !libc::localtime_r(&ts, out.as_mut_ptr()).is_null() };

    #[cfg(windows)]
    let ok = unsafe { libc::localtime_s(out.as_mut_ptr(), &ts) == 0 };

    if !ok {
        return "--:--".to_string();
    }

    let tm = unsafe { out.assume_init() };
    let hour = tm.tm_hour;
    let minute = tm.tm_min;

    format!("{:02}:{:02}", hour, minute)
}
