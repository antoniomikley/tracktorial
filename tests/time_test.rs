#[test]
fn time_parse_as_hms() {
    let parsed_time = tracktorial::time::parse_date_time("18:30:45").unwrap();
    let naive_time = chrono::NaiveTime::from_hms_opt(18, 30, 48).unwrap();
    assert_eq!(
        chrono::Local::now().with_time(naive_time).unwrap(),
        parsed_time
    );
}
#[test]
fn time_parse_as_hm() {
    let parsed_time = tracktorial::time::parse_date_time("18:30").unwrap();
    let naive_time = chrono::NaiveTime::from_hms_opt(18, 30, 0).unwrap();
    assert_eq!(
        chrono::Local::now().with_time(naive_time).unwrap(),
        parsed_time
    );
}
#[test]
fn time_parse_as_h() {
    let parsed_time = tracktorial::time::parse_date_time("18").unwrap();
    let naive_time = chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap();
    assert_eq!(
        chrono::Local::now().with_time(naive_time).unwrap(),
        parsed_time
    );
}
