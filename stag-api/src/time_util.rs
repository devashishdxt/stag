//! Time related utilities.
#[cfg(feature = "wasm")]
use anyhow::anyhow;
use anyhow::Result;
#[cfg(feature = "wasm")]
use time::Month;
use time::OffsetDateTime;

#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
/// Returns current date time in UTC
pub fn now_utc() -> Result<OffsetDateTime> {
    Ok(OffsetDateTime::now_utc())
}

#[cfg(feature = "wasm")]
/// Returns current date time in UTC
pub fn now_utc() -> Result<OffsetDateTime> {
    use time::PrimitiveDateTime;

    let date = js_sys::Date::new_0();

    let primitive_date = time::Date::from_calendar_date(
        date.get_utc_full_year() as i32,
        get_month(date.get_utc_month())?,
        date.get_utc_date() as u8,
    )?;

    let primitive_time = time::Time::from_hms_milli(
        date.get_utc_hours() as u8,
        date.get_utc_minutes() as u8,
        date.get_utc_seconds() as u8,
        date.get_utc_milliseconds() as u16,
    )?;

    Ok(PrimitiveDateTime::new(primitive_date, primitive_time).assume_utc())
}

#[cfg(feature = "wasm")]
fn get_month(month: u32) -> Result<Month> {
    match month {
        0 => Ok(Month::January),
        1 => Ok(Month::February),
        2 => Ok(Month::March),
        3 => Ok(Month::April),
        4 => Ok(Month::May),
        5 => Ok(Month::June),
        6 => Ok(Month::July),
        7 => Ok(Month::August),
        8 => Ok(Month::September),
        9 => Ok(Month::October),
        10 => Ok(Month::November),
        11 => Ok(Month::December),
        _ => Err(anyhow!("invalid month")),
    }
}
