/// Pure calendar arithmetic helpers for `JsDate`.
///
/// These functions live in a separate module so `JsDate`'s main implementation
/// stays focused on behaviour (getters/setters/formatting) while all the
/// year/month/day math is isolated here.
pub(crate) fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub(crate) fn days_in_month(year: i64, month: i32) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

pub(crate) fn days_since_epoch(year: i64, month: i32, day: i64) -> i64 {
    let y = year;
    let mut m = month;
    m = m.clamp(0, 11);

    let mut total_days = 0i64;

    for yr in 1970..y {
        total_days += if is_leap_year(yr) { 366 } else { 365 };
    }
    for yr in y..1970 {
        total_days -= if is_leap_year(yr) { 366 } else { 365 };
    }

    for mo in 0..m {
        total_days += days_in_month(y, mo + 1);
    }

    total_days += day - 1;
    total_days
}

pub(crate) fn civil_from_days(days: i64) -> (i64, i32, i64) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe.div_euclid(1460) + doe.div_euclid(36524) - doe.div_euclid(146096))
        .div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (yoe * 365) - yoe.div_euclid(4) + yoe.div_euclid(100);
    let mp = (5 * doy + 2).div_euclid(153);
    let d = doy - (mp * 153 + 2).div_euclid(5) + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m as i32, d)
}

pub(crate) fn date_from_millis(ms: f64) -> (i64, i32, i64) {
    let days = (ms / 86400000.0).floor() as i64;
    let (year, month, day) = civil_from_days(days);
    (year, month, day)
}

pub(crate) fn parse_iso8601(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.len() < 10 {
        return None;
    }

    let bytes = s.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }

    let year: i64 = s[0..4].parse().ok()?;
    let month: i32 = s[5..7].parse().ok()?;
    let day: i64 = s[8..10].parse().ok()?;

    if s.len() == 10 {
        let days = days_since_epoch(year, month - 1, day);
        return Some(days as f64 * 86400000.0);
    }

    if s.len() < 19 {
        return None;
    }
    if bytes[10] != b'T' && bytes[10] != b' ' {
        return None;
    }
    if bytes[13] != b':' || bytes[16] != b':' {
        return None;
    }

    let hours: f64 = s[11..13].parse().ok()?;
    let minutes: f64 = s[14..16].parse().ok()?;
    let seconds: f64 = s[17..19].parse().ok()?;

    let mut ms = 0.0f64;
    let mut tz_offset = 0i64;

    let rest = &s[19..];
    if let Some(frac_str) = rest.strip_prefix('.') {
        let frac_end = frac_str
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(frac_str.len());
        let frac_val: f64 = format!("0.{}", &frac_str[..frac_end])
            .parse()
            .unwrap_or(0.0);
        ms = frac_val * 1000.0;
        let rest = &rest[1 + frac_end..];
        if rest == "Z" || rest.is_empty() {
            tz_offset = 0;
        } else if (rest.starts_with('+') || rest.starts_with('-')) && rest.len() >= 6 {
            let sign = if rest.starts_with('-') { -1 } else { 1 };
            let tz_h: i64 = rest[1..3].parse().ok()?;
            let tz_m: i64 = rest[4..6].parse().ok()?;
            tz_offset = sign * (tz_h * 60 + tz_m);
        }
    } else if rest == "Z" || rest.is_empty() {
        tz_offset = 0;
    } else if (rest.starts_with('+') || rest.starts_with('-')) && rest.len() >= 6 {
        let sign = if rest.starts_with('-') { -1 } else { 1 };
        let tz_h: i64 = rest[1..3].parse().ok()?;
        let tz_m: i64 = rest[4..6].parse().ok()?;
        tz_offset = sign * (tz_h * 60 + tz_m);
    }

    let days = days_since_epoch(year, month - 1, day);
    let result =
        days as f64 * 86400000.0 + hours * 3600000.0 + minutes * 60000.0 + seconds * 1000.0 + ms
            - tz_offset as f64 * 60000.0;

    Some(result)
}
