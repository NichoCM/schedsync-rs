use dotenv::dotenv;

pub struct TestConfig {
    pub test_ical_username: String,
    pub test_ical_password: String
}

impl TestConfig {

    pub fn new() -> Self {
        dotenv().ok();
        Self {
            test_ical_username: std::env::var("TEST_ICAL_USERNAME").unwrap(),
            test_ical_password: std::env::var("TEST_ICAL_PASSWORD").unwrap(),
        }
    }

}