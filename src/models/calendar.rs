pub struct Calendar {
    pub id: u64,
    pub external_id: String,
    pub name: String,
    pub background_color: String,
    pub foreground_color: String,
    pub integration_id: String,
}

/**
 * This is the intermediate struct that is used to map the JSON response from any calendar
 * service. When syncing calendars, we will compare the external_id to determine if the calendar
 * already exists in the database, and if it does, we will update the calendar with the new data.
 * If the calendar does not exist, we will create a new calendar with the actual Calendar struct.
 */
#[derive(Debug)]
pub struct CalendarResult {
    pub external_id: String,
    pub name: String,
    pub background_color: String,
    pub foreground_color: String,
}