use tracktorial::{api::FactorialApi, config::Configuration, login::Credential};

#[test]
fn can_retrieve_config_file() {
    let config = Configuration::get_config();
    assert_eq!(true, config.is_ok());
}

#[test]
fn config_with_only_email_gets_repopulated_on_login() {
    let mut minimal_config = Configuration::default();
    let my_config = Configuration::get_config().unwrap();
    minimal_config.email = my_config.email.clone();
    let cred = Credential::new_without_password(&minimal_config.email);
    FactorialApi::new(cred, &mut minimal_config).unwrap();
    assert_eq!(my_config, minimal_config);
}
