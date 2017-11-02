use std::error::Error;
use chrono;
use chrono::NaiveDateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use DateTime;

pub fn parse_date(date : &str) -> Result<DateTime, Box<Error>> {
    let mut res : Vec<u32> = Vec::new();
    for element in date.split_whitespace() {
        res.push(element.parse()?);
    }
    let date = match (res.get(0), res.get(1), res.get(2)) {
        (Some(&d), Some(&m), Some(&y)) => {
            NaiveDate::from_ymd_opt(y as i32, m, d)
        }, _ => {Err("Year, month or day not specified!")?; None}
    }.ok_or("Invalid date!")?;
    let time = match (res.get(3), res.get(4), res.get(5)) {
        (None, _, _) => NaiveTime::from_hms_opt(23, 59, 59),
        (Some(&h), None, _) => NaiveTime::from_hms_opt(h, 59, 59),
        (Some(&h), Some(&m), None) => NaiveTime::from_hms_opt(h, m, 59),
        (Some(&h), Some(&m), Some(&s)) => NaiveTime::from_hms_opt(h, m, s),
    }.ok_or("Invalid time!")?;
    Ok(DateTime::from_utc(NaiveDateTime::new(date, time), chrono::Utc))
}
