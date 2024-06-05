use tracktorial::config::Configuration;

#[test]
fn can_retrieve_config_file() {
    let config = Configuration::get_config();
    assert_eq!(true, config.is_ok());
}
