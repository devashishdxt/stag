//! Time related utilities.
use chrono::{DateTime, Utc};
#[cfg(feature = "wasm")]
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
#[cfg_attr(coverage, no_coverage)]
/// Returns current date time in UTC
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(feature = "wasm")]
/// Returns current date time in UTC
pub fn now_utc() -> DateTime<Utc> {
    let date = js_sys::Date::new_0();

    let naive_date = NaiveDate::from_ymd_opt(
        date.get_utc_full_year() as i32,
        date.get_utc_month() + 1,
        date.get_utc_date(),
    )
    .unwrap();

    let naive_time = NaiveTime::from_hms_milli_opt(
        date.get_utc_hours(),
        date.get_utc_minutes(),
        date.get_utc_seconds(),
        date.get_utc_milliseconds(),
    )
    .unwrap();

    let naive_date_time = NaiveDateTime::new(naive_date, naive_time);

    DateTime::<Utc>::from_utc(naive_date_time, Utc)
}
