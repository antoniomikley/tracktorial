use chrono::{DateTime, Datelike, Days, Local, Weekday};
use tracktorial::{api::FactorialApi, config::Configuration, login::Credential};

#[test]
fn client_authentication_with_invalid_cred() {
    let invalid_cred = Credential::new("", "");
    let config = Configuration::get_config().unwrap();
    let api = FactorialApi::new(invalid_cred, config);
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
    let api = FactorialApi::new(valid_cred, config);
    assert_eq!(true, api.is_ok());
}

#[test]
fn starting_shift() {
    let sunday = get_next_sunday();
    let mut config = Configuration::get_config().unwrap();
    let api = FactorialApi::get_api(&mut config).unwrap();
    api.delete_all_shifts(sunday).unwrap();
    let result = api.shift_start(sunday);
    match result.as_ref() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e.to_string()),
    }
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
