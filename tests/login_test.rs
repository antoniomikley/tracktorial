use tracktorial::{config::Configuration, login::Credential};

#[test]
fn test_client_authentication_with_invalid_cred() {
    let invalid_cred = Credential::new("", "");
    match invalid_cred.authenticate_client() {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    };
}

#[test]
fn test_client_authentication_with_valid_cred() {
    let mut config = Configuration::get_config().unwrap();
    if config.email.len() == 0 {
        config.prompt_for_email().unwrap();
        config.write_config().unwrap();
    }
    let mut valid_cred = Credential::new_without_password(&config.email);
    if valid_cred.get_password().is_err() {
        valid_cred.ask_for_password().unwrap();
    }
    match valid_cred.authenticate_client() {
        Ok(_) => assert!(true),
        Err(_) => assert!(false),
    };
}
