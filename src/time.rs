use anyhow::anyhow;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime};
use rand::Rng;

/// A day where no work has to be done.
#[derive(Debug, Clone)]
pub struct FreeDay {
    /// The day without work
    pub day: DateTime<Local>,
    /// The part of the day without work
    pub half: HalfDay,
}

impl PartialOrd for FreeDay {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for FreeDay {}
impl Ord for FreeDay {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.day.cmp(&other.day)
    }
}
impl PartialEq<DateTime<Local>> for FreeDay {
    fn eq(&self, other: &DateTime<Local>) -> bool {
        self.day.to_string() == *other.to_string()
    }
}
impl PartialEq for FreeDay {
    fn eq(&self, other: &Self) -> bool {
        self.day.to_string() == other.day.to_string()
    }
}
impl From<DateTime<Local>> for FreeDay {
    fn from(value: DateTime<Local>) -> Self {
        FreeDay {
            day: value,
            half: HalfDay::WholeDay,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HalfDay {
    StartOfDay,
    EndOfDay,
    WholeDay,
}

/// Get a chrono::Duration from a &str in the format of <hours>h<minutes>m<seconds>.
pub fn parse_duration(time: &str) -> anyhow::Result<Duration> {
    let mut time = String::from(time);
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
    if !formatter.contains('h') {
        formatter.push_str("%Hh");
        time.push_str("0h");
    }
    if !formatter.contains('m') {
        formatter.push_str("%Mm");
        time.push_str("0m");
    }
    if !formatter.contains('s') {
        formatter.push_str("%Ss");
        time.push_str("0s");
    }
    let naive_time = NaiveTime::parse_from_str(&time, &formatter).expect("here?");
    let duration = naive_time.signed_duration_since(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    Ok(duration)
}

/// Get a chrono::DateTime<Local> from a &str with the format of either
/// year-month-dayThours:minutes:seconds or HH:MM:SS if the desired date is today.
pub fn parse_date_time(date_time: &str) -> anyhow::Result<DateTime<Local>> {
    let today = Local::now();
    if date_time.contains('T') {
        let date_time_with_offset = format!("{date_time}{}", today.offset().to_string());
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

/// Get a chrono::DateTime<Local> from a &str with the format YYYY-mm-dd or dd.mm.YYYY. The
/// time at that date will be 00:00:00
pub fn parse_date(date: &str) -> anyhow::Result<DateTime<Local>> {
    let today = Local::now()
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(ymd) => {
            return Ok(today
                .with_year(ymd.year_ce().1.try_into().unwrap())
                .unwrap()
                .with_month(1)
                .unwrap()
                .with_day(ymd.day())
                .unwrap()
                .with_month(ymd.month())
                .unwrap())
        }
        Err(_) => {}
    };
    match NaiveDate::parse_from_str(date, "%d.%m.%Y") {
        Ok(ymd) => {
            return Ok(today
                .with_year(ymd.year_ce().1.try_into().unwrap())
                .unwrap()
                .with_month(1)
                .unwrap()
                .with_day(ymd.day())
                .unwrap()
                .with_month(ymd.month())
                .unwrap())
        }
        Err(_) => {}
    }
    Err(anyhow!("Could not parse date."))
}

/// Get the mandatory duration for a break depending on the duration of work as required by german
/// law.
pub fn get_break_duration(work_duration: chrono::Duration) -> chrono::Duration {
    let mut break_duration = chrono::Duration::new(0, 0).unwrap();
    if work_duration.num_hours() > 6 {
        break_duration = break_duration
            .checked_add(&chrono::Duration::minutes(30))
            .unwrap();
    }
    if work_duration.num_hours() > 8 {
        break_duration = break_duration
            .checked_add(&chrono::Duration::minutes(15))
            .unwrap()
    }
    break_duration
}

/// A day to get work done.
#[derive(Debug)]
pub struct WorkDay {
    /// The date and time to clock in
    pub clock_in: chrono::DateTime<Local>,
    /// The date and time to take a break
    pub break_start: chrono::DateTime<Local>,
    /// The date and time to end the break
    pub break_end: chrono::DateTime<Local>,
    /// The date and time to stop working
    pub clock_out: chrono::DateTime<Local>,
}
impl WorkDay {
    /// Apply
    pub fn randomize_shift(
        start: chrono::DateTime<Local>,
        duration: chrono::Duration,
        max_rand_range: u16,
    ) -> Self {
        let start_offset: i32 =
            rand::thread_rng().gen_range(max_rand_range as i32 * -60..=max_rand_range as i32 * 60);
        let break_offset: i32 =
            rand::thread_rng().gen_range(max_rand_range as i32 * -60..=max_rand_range as i32 * 60);

        let clock_in = start
            .checked_add_signed(Duration::seconds(start_offset.into()))
            .unwrap();
        let first_shift_len = duration
            .checked_div(2)
            .unwrap()
            .checked_add(&Duration::seconds(break_offset.into()))
            .unwrap();
        let second_shift_len = duration.checked_sub(&first_shift_len).unwrap();
        let break_start = clock_in.checked_add_signed(first_shift_len).unwrap();
        let break_end = break_start
            .checked_add_signed(get_break_duration(duration))
            .unwrap();
        let clock_out = break_end.checked_add_signed(second_shift_len).unwrap();

        WorkDay {
            clock_in,
            break_start,
            break_end,
            clock_out,
        }
    }
    pub fn standard_shift(start: chrono::DateTime<Local>, duration: chrono::Duration) -> Self {
        let clock_in = start;
        let break_start = start
            .checked_add_signed(duration.checked_div(2).unwrap())
            .unwrap();
        let break_end = break_start
            .checked_add_signed(get_break_duration(duration))
            .unwrap();
        let clock_out = break_end
            .checked_add_signed(duration.checked_div(2).unwrap())
            .unwrap();
        WorkDay {
            clock_in,
            break_start,
            break_end,
            clock_out,
        }
    }
}
