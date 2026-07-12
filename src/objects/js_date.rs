use crate::objects::js_date_calendar;
use std::time::{SystemTime, UNIX_EPOCH};

/// JavaScript Date object representation
/// Stores UTC milliseconds since Unix epoch as f64 (matching JS spec)
#[derive(Debug, Clone)]
pub struct JsDate {
    pub utc_ms: f64,
}

impl JsDate {
    /// Decompose current time into (days, hours_in_day, minutes_in_hour,
    /// seconds_in_minute, milliseconds_in_second) so callers can replace
    /// any component and reassemble via `set_ymdhms`.
    fn components(&self) -> (f64, f64, f64, f64, f64) {
        let days = (self.utc_ms / 86400000.0).floor();
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        let hours = (ms_in_day / 3600000.0).floor();
        let ms_in_hour = ((ms_in_day % 3600000.0) + 3600000.0) % 3600000.0;
        let minutes = (ms_in_hour / 60000.0).floor();
        let ms_in_min = ((ms_in_day % 60000.0) + 60000.0) % 60000.0;
        let seconds = (ms_in_min / 1000.0).floor();
        let millis = ((self.utc_ms % 1000.0) + 1000.0) % 1000.0;
        (days, hours, minutes, seconds, millis)
    }

    /// Reassemble from (days, hours, minutes, seconds, millis) into epoch ms.
    fn set_ymdhms(&mut self, days: f64, h: f64, m: f64, s: f64, ms: f64) {
        self.utc_ms = days * 86400000.0 + h * 3600000.0 + m * 60000.0 + s * 1000.0 + ms;
    }
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
        let mut year = y as i64;
        let month = m as i64;

        if (0..=99).contains(&year) {
            year += 1900;
        }

        let days = js_date_calendar::days_since_epoch(year, month as i32, d as i64);
        let utc_ms = days as f64 * 86400000.0 + h * 3600000.0 + min * 60000.0 + s * 1000.0 + ms;
        JsDate { utc_ms }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        js_date_calendar::parse_iso8601(s).map(|utc_ms| JsDate { utc_ms })
    }

    pub fn is_valid(&self) -> bool {
        self.utc_ms.is_finite()
    }

    // UTC getters
    pub fn get_utc_full_year(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let (y, _, _) = js_date_calendar::date_from_millis(self.utc_ms);
        y as f64
    }

    pub fn get_utc_month(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let (_, m, _) = js_date_calendar::date_from_millis(self.utc_ms);
        (m - 1) as f64
    }

    pub fn get_utc_date(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let (_, _, d) = js_date_calendar::date_from_millis(self.utc_ms);
        d as f64
    }

    pub fn get_utc_day(&self) -> f64 {
        if !self.is_valid() {
            return f64::NAN;
        }
        let days = (self.utc_ms / 86400000.0).floor() as i64;
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

    // Local time delegates to UTC (timezone_offset = 0 for now)
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

    // Setters

    pub fn set_time(&mut self, ms: f64) -> f64 {
        self.utc_ms = ms;
        ms
    }

    pub fn set_utc_full_year(&mut self, y: f64, m: Option<f64>, d: Option<f64>) -> f64 {
        let (_, old_m, old_d) = js_date_calendar::date_from_millis(self.utc_ms);
        let month = m.unwrap_or(old_m as f64 - 1.0) as i32;
        let day = d.unwrap_or(old_d as f64) as i64;
        let days = js_date_calendar::days_since_epoch(y as i64, month, day);
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        self.utc_ms = days as f64 * 86400000.0 + ms_in_day;
        self.utc_ms
    }

    pub fn set_utc_month(&mut self, m: f64, d: Option<f64>) -> f64 {
        let (y, _, old_d) = js_date_calendar::date_from_millis(self.utc_ms);
        let day = d.unwrap_or(old_d as f64) as i64;
        let days = js_date_calendar::days_since_epoch(y, m as i32, day);
        let ms_in_day = ((self.utc_ms % 86400000.0) + 86400000.0) % 86400000.0;
        self.utc_ms = days as f64 * 86400000.0 + ms_in_day;
        self.utc_ms
    }

    pub fn set_utc_date(&mut self, d: f64) -> f64 {
        let (y, m, _) = js_date_calendar::date_from_millis(self.utc_ms);
        let days = js_date_calendar::days_since_epoch(y, m, d as i64);
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
        let (days, _, minutes, seconds, millis) = self.components();
        self.set_ymdhms(
            days,
            h,
            min.unwrap_or(minutes),
            s.unwrap_or(seconds),
            ms.unwrap_or(millis),
        );
        self.utc_ms
    }

    pub fn set_utc_minutes(&mut self, min: f64, s: Option<f64>, ms: Option<f64>) -> f64 {
        let (days, hours, _, seconds, millis) = self.components();
        self.set_ymdhms(days, hours, min, s.unwrap_or(seconds), ms.unwrap_or(millis));
        self.utc_ms
    }

    pub fn set_utc_seconds(&mut self, s: f64, ms: Option<f64>) -> f64 {
        let (days, hours, minutes, _, millis) = self.components();
        self.set_ymdhms(days, hours, minutes, s, ms.unwrap_or(millis));
        self.utc_ms
    }

    pub fn set_utc_milliseconds(&mut self, ms: f64) -> f64 {
        let (days, hours, minutes, seconds, _) = self.components();
        self.set_ymdhms(days, hours, minutes, seconds, ms);
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
        let (y, m, d) = js_date_calendar::date_from_millis(self.utc_ms);
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
        let (y, m, d) = js_date_calendar::date_from_millis(self.utc_ms);
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
        let (y, m, d) = js_date_calendar::date_from_millis(self.utc_ms);
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

    // Locale string representations. The runtime treats local time as UTC
    // (see `get_timezone_offset`), so these mirror the UTC-based formatting.
    pub fn to_locale_time_string(&self) -> String {
        if !self.is_valid() {
            return "Invalid Date".to_string();
        }
        let h = self.get_utc_hours() as u32;
        let min = self.get_utc_minutes() as u32;
        let s = self.get_utc_seconds() as u32;
        format!("{:02}:{:02}:{:02}", h, min, s)
    }

    pub fn to_locale_date_string(&self) -> String {
        if !self.is_valid() {
            return "Invalid Date".to_string();
        }
        let (y, m, d) = js_date_calendar::date_from_millis(self.utc_ms);
        format!("{:04}-{:02}-{:02}", y, m, d)
    }

    pub fn to_locale_string(&self) -> String {
        if !self.is_valid() {
            return "Invalid Date".to_string();
        }
        format!("{} {}", self.to_locale_date_string(), self.to_locale_time_string())
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
