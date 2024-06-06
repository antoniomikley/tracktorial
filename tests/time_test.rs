use chrono::Datelike;
use tracktorial::time::{get_break_duration, parse_date, parse_date_time, parse_duration};

#[test]
fn time_parse_as_hms() {
    let parsed_time = parse_date_time("18:30:45").unwrap();
    let naive_time = chrono::NaiveTime::from_hms_opt(18, 30, 45).unwrap();
    assert_eq!(
        chrono::Local::now().with_time(naive_time).unwrap(),
        parsed_time
    );
}

#[test]
fn time_parse_as_hm() {
    let parsed_time = parse_date_time("18:30").unwrap();
    let naive_time = chrono::NaiveTime::from_hms_opt(18, 30, 0).unwrap();
    assert_eq!(
        chrono::Local::now().with_time(naive_time).unwrap(),
        parsed_time
    );
}

#[test]
fn time_parse_as_h() {
    let parsed_time = parse_date_time("18").unwrap();
    let naive_time = chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap();
    assert_eq!(
        chrono::Local::now().with_time(naive_time).unwrap(),
        parsed_time
    );
}

#[test]
fn date_parse_as_ddmmyyyy() {
    let parsed_date = parse_date("01.01.2024").unwrap();
    let first_jan = chrono::Local::now()
        .with_day(1)
        .unwrap()
        .with_month(1)
        .unwrap()
        .with_year(2024)
        .unwrap()
        .with_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    assert_eq!(first_jan, parsed_date);
}

#[test]
fn date_parse_as_yyyymmdd() {
    let parsed_date = parse_date("2024-01-01").unwrap();
    let first_jan = chrono::Local::now()
        .with_day(1)
        .unwrap()
        .with_month(1)
        .unwrap()
        .with_year(2024)
        .unwrap()
        .with_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    assert_eq!(first_jan, parsed_date);
}

#[test]
fn duration_parse_as_hms() {
    let parsed_duration = parse_duration("2h32m45s").unwrap();
    let duration =
        chrono::Duration::hours(2) + chrono::Duration::minutes(32) + chrono::Duration::seconds(45);
    assert_eq!(duration, parsed_duration);
}

#[test]
fn duration_parse_as_hm() {
    let parsed_duration = parse_duration("2h32m").unwrap();
    let duration = chrono::Duration::hours(2) + chrono::Duration::minutes(32);
    assert_eq!(duration, parsed_duration);
}

#[test]
fn duration_parse_as_h() {
    let parsed_duration = parse_duration("2h").unwrap();
    let duration = chrono::Duration::hours(2);
    assert_eq!(duration, parsed_duration);
}
#[test]
fn duration_parse_as_m() {
    let parsed_duration = parse_duration("30m").unwrap();
    let duration = chrono::Duration::minutes(30);
    assert_eq!(duration, parsed_duration);
}

#[test]
fn duration_as_m_cannot_overflow_into_h() {
    let parsed_duration = parse_duration("120m");
    assert!(parsed_duration.is_err());
}

#[test]
fn no_break_if_working_less_than_six_h() {
    let break_duration = get_break_duration(parse_duration("5h59m").unwrap());
    assert_eq!(chrono::Duration::minutes(0), break_duration);
}

#[test]
fn thirty_min_break_if_working_more_than_six_h() {
    let break_duration = get_break_duration(parse_duration("6h").unwrap());
    assert_eq!(chrono::Duration::minutes(30), break_duration);
    let break_duration = get_break_duration(parse_duration("8h59m").unwrap());
    assert_eq!(chrono::Duration::minutes(30), break_duration);
}

#[test]
fn thirty_min_break_if_working_more_than_nine_h() {
    let break_duration = get_break_duration(parse_duration("9h").unwrap());
    assert_eq!(chrono::Duration::minutes(45), break_duration);
}
