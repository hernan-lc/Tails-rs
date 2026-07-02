use std::time::{SystemTime, UNIX_EPOCH};

/// JavaScript Date object representation
/// Stores UTC milliseconds since Unix epoch as f64 (matching JS spec)
#[derive(Debug, Clone)]
pub struct JsDate {
    pub utc_ms: f64,
}

impl JsDate {
    pub fn now() -> Self {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;
        JsDate { utc_ms: ms }
    }

    pub fn from_millis(ms: f64) -> Self {
        JsDate { utc_ms: ms }
    }

    pub fn from_components(y: f64, m: f64, d: f64, h: f64, min: f64, s: f64, ms: f64) -> Self {
        // Use simple UTC-based calculation
        let mut year = y as i64;
        let month = m as i64;

        // If year is 0-99, treat as 1900+year (JS spec behavior)
        if (0..=99).contains(&year) {
            year += 1900;
        }

        let days = days_since_epoch(year, month as i32, d as i64);
        let utc_ms = days as f64 * 86400000.0 + h * 3600000.0 + min * 60000.0 + s * 1000.0 + ms;
        JsDate { utc_ms }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        // Try ISO 8601 format: YYYY-MM-DDTHH:mm:ss.sssZ
        if let Some(ms) = parse_iso8601(s) {
            return Some(JsDate { utc_ms: ms });
        }
        // Try other common formats
        None
    }

    pub fn is_valid(&self) -> bool {
        self.utc_ms.is_finite()
    }

    // UTC getters
    pub fn get_utc_full_year(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let (y, _, _) = date_from_millis(self.utc_ms);
        y as f64
    }

    pub fn get_utc_month(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let (_, m, _) = date_from_millis(self.utc_ms);
        (m - 1) as f64
    }

    pub fn get_utc_date(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let (_, _, d) = date_from_millis(self.utc_ms);
        d as f64
    }

    pub fn get_utc_day(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let days = (self.utc_ms / 86400000.0).floor() as i64;
        // Jan 1 1970 was a Thursday (4)
        ((days % 7 + 4) % 7) as f64
    }

    pub fn get_utc_hours(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        (ms_in_day / 3600000.0).floor()
    }

    pub fn get_utc_minutes(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let ms_in_hour = ((self.utc_ms % 3600000.0) + 3600000.0) % 3600000.0;
        (ms_in_hour / 60000.0).floor()
    }

    pub fn get_utc_seconds(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let ms_in_min = ((self.utc_ms % 60000.0) + 60000.0) % 60000.0;
        (ms_in_min / 1000.0).floor()
    }

    pub fn get_utc_milliseconds(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        ((self.utc_ms % 1000.0) + 1000.0) % 1000.0
    }

    // For local time, we use UTC with a simple offset (0 for now)
    // In a real implementation, this would use the system timezone
    pub fn get_full_year(&self) -> f64 {
        self.get_utc_full_year()
    }
    pub fn get_month(&self) -> f64 {
        self.get_utc_month()
    }
    pub fn get_date(&self) -> f64 {
        self.get_utc_date()
    }
    pub fn get_day(&self) -> f64 {
        self.get_utc_day()
    }
    pub fn get_hours(&self) -> f64 {
        self.get_utc_hours()
    }
    pub fn get_minutes(&self) -> f64 {
        self.get_utc_minutes()
    }
    pub fn get_seconds(&self) -> f64 {
        self.get_utc_seconds()
    }
    pub fn get_milliseconds(&self) -> f64 {
        self.get_utc_milliseconds()
    }
    pub fn get_timezone_offset(&self) -> f64 {
        0.0
    }

    // Setters (modify in place, return new ms)
    pub fn set_time(&mut self, ms: f64) -> f64 {
        self.utc_ms = ms;
        ms
    }

    pub fn set_utc_full_year(&mut self, y: f64, m: Option<f64>, d: Option<f64>) -> f64 {
        let (_, old_m, old_d) = date_from_millis(self.utc_ms);
        let month = m.unwrap_or(old_m as f64 - 1.0) as i32;
        let day = d.unwrap_or(old_d as f64) as i64;
        let days = days_since_epoch(y as i64, month, day);
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        self.utc_ms = days as f64 * 86400000.0 + ms_in_day;
        self.utc_ms
    }

    pub fn set_utc_month(&mut self, m: f64, d: Option<f64>) -> f64 {
        let (y, _, old_d) = date_from_millis(self.utc_ms);
        let day = d.unwrap_or(old_d as f64) as i64;
        let days = days_since_epoch(y, m as i32, day);
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        self.utc_ms = days as f64 * 86400000.0 + ms_in_day;
        self.utc_ms
    }

    pub fn set_utc_date(&mut self, d: f64) -> f64 {
        let (y, m, _) = date_from_millis(self.utc_ms);
        let days = days_since_epoch(y, m, d as i64);
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        self.utc_ms = days as f64 * 86400000.0 + ms_in_day;
        self.utc_ms
    }

    pub fn set_utc_hours(
        &mut self,
        h: f64,
        min: Option<f64>,
        s: Option<f64>,
        ms: Option<f64>,
    ) -> f64 {
        let days = (self.utc_ms / 86400000.0).floor();
        let minutes = min.unwrap_or(self.get_utc_minutes());
        let seconds = s.unwrap_or(self.get_utc_seconds());
        let millis = ms.unwrap_or(self.get_utc_milliseconds());
        self.utc_ms =
            days * 86400000.0 + h * 3600000.0 + minutes * 60000.0 + seconds * 1000.0 + millis;
        self.utc_ms
    }

    pub fn set_utc_minutes(&mut self, min: f64, s: Option<f64>, ms: Option<f64>) -> f64 {
        let days = (self.utc_ms / 86400000.0).floor();
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        let hours = (ms_in_day / 3600000.0).floor();
        let seconds = s.unwrap_or(self.get_utc_seconds());
        let millis = ms.unwrap_or(self.get_utc_milliseconds());
        self.utc_ms =
            days * 86400000.0 + hours * 3600000.0 + min * 60000.0 + seconds * 1000.0 + millis;
        self.utc_ms
    }

    pub fn set_utc_seconds(&mut self, s: f64, ms: Option<f64>) -> f64 {
        let days = (self.utc_ms / 86400000.0).floor();
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        let hours = (ms_in_day / 3600000.0).floor();
        let ms_in_hour = ((ms_in_day % 3600000.0) + 3600000.0) % 3600000.0;
        let minutes = (ms_in_hour / 60000.0).floor();
        let millis = ms.unwrap_or(self.get_utc_milliseconds());
        self.utc_ms =
            days * 86400000.0 + hours * 3600000.0 + minutes * 60000.0 + s * 1000.0 + millis;
        self.utc_ms
    }

    pub fn set_utc_milliseconds(&mut self, ms: f64) -> f64 {
        let days = (self.utc_ms / 86400000.0).floor();
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        let hours = (ms_in_day / 3600000.0).floor();
        let ms_in_hour = ((ms_in_day % 3600000.0) + 3600000.0) % 3600000.0;
        let minutes = (ms_in_hour / 60000.0).floor();
        let ms_in_min = ((ms_in_day % 60000.0) + 60000.0) % 60000.0;
        let seconds = (ms_in_min / 1000.0).floor();
        self.utc_ms =
            days * 86400000.0 + hours * 3600000.0 + minutes * 60000.0 + seconds * 1000.0 + ms;
        self.utc_ms
    }

    // Local setters delegate to UTC for now
    pub fn set_full_year(&mut self, y: f64, m: Option<f64>, d: Option<f64>) -> f64 {
        self.set_utc_full_year(y, m, d)
    }
    pub fn set_month(&mut self, m: f64, d: Option<f64>) -> f64 {
        self.set_utc_month(m, d)
    }
    pub fn set_date(&mut self, d: f64) -> f64 {
        self.set_utc_date(d)
    }
    pub fn set_hours(&mut self, h: f64, min: Option<f64>, s: Option<f64>, ms: Option<f64>) -> f64 {
        self.set_utc_hours(h, min, s, ms)
    }
    pub fn set_minutes(&mut self, min: f64, s: Option<f64>, ms: Option<f64>) -> f64 {
        self.set_utc_minutes(min, s, ms)
    }
    pub fn set_seconds(&mut self, s: f64, ms: Option<f64>) -> f64 {
        self.set_utc_seconds(s, ms)
    }
    pub fn set_milliseconds(&mut self, ms: f64) -> f64 {
        self.set_utc_milliseconds(ms)
    }

    // String representations
    pub fn to_iso_string(&self) -> String {
        if !self.is_valid() {
            return "Invalid Date".to_string();
        }
        let (y, m, d) = date_from_millis(self.utc_ms);
        let h = self.get_utc_hours() as u32;
        let min = self.get_utc_minutes() as u32;
        let s = self.get_utc_seconds() as u32;
        let ms = self.get_utc_milliseconds() as u32;
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            y, m, d, h, min, s, ms
        )
    }

    pub fn to_utc_string(&self) -> String {
        if !self.is_valid() {
            return "Invalid Date".to_string();
        }
        let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let (y, m, d) = date_from_millis(self.utc_ms);
        let day_idx = self.get_utc_day() as usize;
        let h = self.get_utc_hours() as u32;
        let min = self.get_utc_minutes() as u32;
        let s = self.get_utc_seconds() as u32;
        format!(
            "{} {:02} {} {} {:02}:{:02}:{:02} GMT",
            days[day_idx],
            d,
            months[(m - 1) as usize],
            y,
            h,
            min,
            s
        )
    }

    pub fn to_date_string(&self) -> String {
        if !self.is_valid() {
            return "Invalid Date".to_string();
        }
        let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let (y, m, d) = date_from_millis(self.utc_ms);
        let day_idx = self.get_utc_day() as usize;
        format!("{} {} {} {}", days[day_idx], months[(m - 1) as usize], d, y)
    }

    pub fn to_time_string(&self) -> String {
        if !self.is_valid() {
            return "Invalid Date".to_string();
        }
        let h = self.get_utc_hours() as u32;
        let min = self.get_utc_minutes() as u32;
        let s = self.get_utc_seconds() as u32;
        format!("{:02}:{:02}:{:02} GMT+0000", h, min, s)
    }
}

impl std::fmt::Display for JsDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_valid() {
            write!(f, "{}", self.to_utc_string())
        } else {
            write!(f, "Invalid Date")
        }
    }
}

// Helper functions

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i64, month: i32) -> i64 {
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

fn days_since_epoch(year: i64, month: i32, day: i64) -> i64 {
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

fn civil_from_days(days: i64) -> (i64, i32, i64) {
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

fn date_from_millis(ms: f64) -> (i64, i32, i64) {
    let days = (ms / 86400000.0).floor() as i64;
    let (year, month, day) = civil_from_days(days);
    (year, month, day)
}

fn parse_iso8601(s: &str) -> Option<f64> {
    // Parse YYYY-MM-DDTHH:mm:ss.sssZ
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
        // Date only
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
    let mut tz_offset = 0i64; // minutes from UTC

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
