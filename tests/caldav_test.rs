use schedsync_api::connectors::caldav;
mod common;

#[tokio::test]
pub async fn caldav_test() {

    let config = common::test_config::TestConfig::new();

    let url = "https://caldav.icloud.com";
    let username =  config.test_ical_username;
    let password = config.test_ical_password;

    let data = caldav::caldav::get_principal(
        url.to_string(),
        username.to_string(),
        Some(password.to_string()),
    ).await.unwrap();

    // Get the calendar
    let result = match caldav::caldav::get_calendar(
        &data,
        url.to_string(),
        username.to_string(),
        Some(password.to_string())
    ).await {
        Ok(result) => result,
        Err(error) => {
            println!("Error {:?}", error);
            assert!(false);
            return;
        }
    };

    for calendar in result.iter() {

        let result = caldav::caldav::get_events(
            &calendar,
            url.to_string(),
            username.to_string(),
            Some(password.to_string())
        ).await;

        match result {
            Ok(result) => {
                println!("SUCCESS {:?}", result);
            },
            Err(_) => {
                // println!("Error {:?}", result);
            }
        }
    }
}