use anyhow::anyhow;
use chrono::{DateTime, Duration, Local, NaiveTime};

pub fn parse_duration(time: &str) -> anyhow::Result<Duration> {
    let mut formatter = String::new();
    if time.contains('h') {
        formatter.push_str("%Hh");
    }
    if time.contains('m') {
        formatter.push_str("%Mm");
    }
    if time.contains('s') {
        formatter.push_str("%Ss");
    }
    let naive_time = NaiveTime::parse_from_str(time, &formatter)?;
    let duration = naive_time.signed_duration_since(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    Ok(duration)
}

pub fn parse_date_time(date_time: &str) -> anyhow::Result<DateTime<Local>> {
    let today = Local::now();
    if date_time.contains('T') {
        let date_time_with_offset = format!("{date_time}{}", today.offset().to_string());
        println!("{date_time_with_offset}");
        let time = DateTime::parse_from_rfc3339(&date_time_with_offset)?;
        return Ok(time.into());
    }
    match NaiveTime::parse_from_str(date_time, "%H:%M:%S") {
        Ok(hms) => return Ok(today.with_time(hms).unwrap()),
        Err(_) => {}
    };
    match NaiveTime::parse_from_str(date_time, "%H:%M") {
        Ok(hm) => return Ok(today.with_time(hm).unwrap()),
        Err(_) => {}
    };
    match NaiveTime::parse_from_str(date_time, "%H") {
        Ok(hours) => return Ok(today.with_time(hours).unwrap()),
        Err(_) => {}
    };
    Err(anyhow!("Could not parse the time."))
}
