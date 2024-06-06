use std::sync::Mutex;

use chrono::{DateTime, Datelike, Days, Local, Timelike, Weekday};
use once_cell::sync::Lazy;
use serial_test::serial;
use tracktorial::{api::FactorialApi, config::Configuration, login::Credential};

static API_MUTEX: Lazy<Mutex<FactorialApi>> = Lazy::new(|| {
    let api = FactorialApi::get_api().unwrap();
    Mutex::new(api)
});

#[test]
fn client_authentication_with_invalid_cred() {
    let invalid_cred = Credential::new("", "");
    let mut config = Configuration::get_config().unwrap();
    let api = FactorialApi::new(invalid_cred, &mut config);
    assert_eq!(true, api.is_err());
}

#[test]
fn client_authentication_with_valid_cred() {
    let mut config = Configuration::get_config().unwrap();
    if config.email.len() == 0 {
        config.prompt_for_email().unwrap();
        config.write_config().unwrap();
    }
    let mut valid_cred = Credential::new_without_password(&config.email);
    if valid_cred.get_password().is_err() {
        valid_cred.ask_for_password().unwrap();
    }
    let api = FactorialApi::new(valid_cred, &mut config);
    assert_eq!(true, api.is_ok());
}

#[serial]
#[test]
fn starting_shift() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    let result = api.shift_start(sunday);
    match result.as_ref() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e.to_string()),
    }
    api.delete_all_shifts(sunday).unwrap();

    assert_eq!(true, result.is_ok());
}

#[serial]
#[test]
fn cannot_clock_in_if_already_clocked_in() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    api.shift_start(sunday).unwrap();
    let result = api.shift_start(sunday);
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_err());
}

#[serial]
#[test]
fn starting_break() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    api.shift_start(sunday).unwrap();
    let result = api.break_start(sunday);
    match result.as_ref() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e.to_string()),
    }
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_ok());
}

#[serial]
#[test]
fn cannot_start_break_if_not_clocked_in() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    let result = api.break_start(sunday);
    assert_eq!(true, result.is_err());
}

#[serial]
#[test]
fn cannot_start_break_if_already_on_break() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    api.shift_start(sunday).unwrap();
    api.break_start(sunday).unwrap();
    let result = api.break_start(sunday);
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_err());
}

#[serial]
#[test]
fn ending_break() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    api.shift_start(sunday).unwrap();
    api.break_start(sunday).unwrap();
    let result = api.break_end(sunday);
    match result.as_ref() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e.to_string()),
    }
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_ok());
}

#[serial]
#[test]
fn cannot_end_break_if_not_on_break() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    api.shift_start(sunday).unwrap();
    let result = api.break_end(sunday);
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_err());
}

#[serial]
#[test]
fn ending_shift() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    api.shift_start(sunday).unwrap();
    let result = api.shift_end(sunday);
    match result.as_ref() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e.to_string()),
    }
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_ok());
}

#[serial]
#[test]
fn create_shift() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    let result = api.make_shift(
        sunday.with_hour(8).unwrap(),
        sunday
            .checked_add_signed(chrono::Duration::hours(8))
            .unwrap(),
    );
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_ok());
}

#[serial]
#[test]
fn create_break() {
    let sunday = get_next_sunday();
    let api = API_MUTEX.lock().unwrap();
    let result = api.make_break(
        sunday.with_hour(8).unwrap(),
        sunday
            .checked_add_signed(chrono::Duration::hours(8))
            .unwrap(),
    );
    api.delete_all_shifts(sunday).unwrap();
    assert_eq!(true, result.is_ok());
}

fn get_next_sunday() -> DateTime<Local> {
    let mut today = Local::now();
    while today.weekday() != Weekday::Sun {
        today = today.checked_add_days(Days::new(1)).unwrap();
    }
    today
}
