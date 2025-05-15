use core::panic;
use std::{collections::HashMap, io::{BufReader, Cursor}};

use ical::{line, parser::ical::component::IcalEvent, property::Property};
use serde::{Deserialize, Serialize};
use quick_xml::{se::Serializer, de::Deserializer, de::DeError};

fn to_xml_string<T: Serialize>(data: &T) -> Result<String, Box<dyn std::error::Error>> {
    // Create a serializer with the writer (Cursor in this case)
    let mut buffer = String::new();
    let serializer = Serializer::new(&mut buffer);

    // Serialize the data to XML
    data.serialize(serializer)?;

    Ok(buffer)
}

fn parse_xml<T: for<'a> Deserialize<'a>>(data: &str) -> Result<T, DeError> {
    quick_xml::de::from_str(data)
}

/**
 * Get the principal data from the CalDAV server.
 */
pub async fn get_principal(
    url: String,
    username: String,
    password: Option<String>
) -> Result<PrincipalData, anyhow::Error> {
    let method = reqwest::Method::from_bytes(b"PROPFIND").unwrap();
    let client = reqwest::Client::new();

    // Serialize the payload
   let Ok(payload) = to_xml_string(&Propfind {
        d: "DAV:".to_string(),
        cal: "urn:ietf:params:xml:ns:caldav".to_string(),
        cs: "http://calendarserver.org/ns/".to_string(),
        prop: PrincipalRequestProp::make(),
    }) else {
        return Err(anyhow::anyhow!("get_principle: Error serializing payload"));
    };
        
    // Send the get principal request
    let Ok(response) = client
        .request(method, url)
        .basic_auth(username, password)
        .body(payload).send().await else {
        return Err(anyhow::anyhow!("get_principle: Error sending request"));
    };

    // Expect a 207 status code
    if response.status() != 207 {
        return Err(anyhow::anyhow!("get_principle: Error status code: {}", response.status()));
    }
    
    // Read the response
    let Ok(text) = response.text().await else {
        return Err(anyhow::anyhow!("get_principle: Error reading response"));
    };

    // Deserialize the response
    let Ok(data) = parse_xml::<MultiStatus<CurrentPrincipleProp>>(&text) else {
        return Err(anyhow::anyhow!("get_principle: Error deserializing response"));
    };

    // Get the first element
    let Some(first) = data.response.first() else {
        return Err(anyhow::anyhow!("get_principle: Error getting first element"));
    };

    // Get the user_id from the path
    let prop = first.propstat.first().unwrap().prop.as_ref().unwrap();
    let path = prop.current_user_principal.href.clone();
    let Some(user_id) = path.trim_matches('/').split("/").nth(0) else {
        return Err(anyhow::anyhow!("get_principle: Error getting user_id"));
    };

    // Return the principal data
    Ok(PrincipalData::new(user_id.to_string(), path))
}

/**
 * Get the calendars from the CalDAV server.
 */
pub async fn get_calendar(
    data: &PrincipalData,
    url: String,
    username: String,
    password: Option<String>
) -> Result<Vec<CaldavCalendar>, anyhow::Error> {
    let method = reqwest::Method::from_bytes(b"PROPFIND").unwrap();
    let client = reqwest::Client::new();

    // Serialize the payload
    let Ok(payload) = to_xml_string(&Propfind {
        d: "DAV:".to_string(),
        cal: "urn:ietf:params:xml:ns:caldav".to_string(),
        cs: "http://calendarserver.org/ns/".to_string(),
        prop: CalendarRequestProp::make(),
    }) else {
        return Err(anyhow::anyhow!("get_calendar: Error serializing payload"));
    };
        
    // Send the get principal request
    let Ok(response) = client
        .request(method, url + "/" + &data.user_id + "/calendars")
        .header("Depth", "1")
        .basic_auth(username, password)
        .body(payload).send().await else {
        return Err(anyhow::anyhow!("get_calendar: Error sending request"));
    };

    // Expect a 207 status code
    if response.status() != 207 {
        return Err(anyhow::anyhow!("get_calendar: Error status code: {}", response.status()));
    }

    // Read the response
    let Ok(text) = response.text().await else {
        return Err(anyhow::anyhow!("get_calendar: Error reading response"));
    };
    
    // Deserialize the response
    let Ok(data) = parse_xml::<MultiStatus<CalendarResponseData>>(&text) else {
        return Err(anyhow::anyhow!("get_calendar: Error deserializing response"));
    };

    // Filter the elements with a 200 status code
    let elements = data.response.iter().filter(|x| {
        let Some(propstat) = x.propstat.first() else {
            return false;
        };
        propstat.status == "HTTP/1.1 200 OK"
    });

    Ok(elements.map(|x| {
        let prop = x.propstat.first().unwrap().prop.as_ref().unwrap();
        CaldavCalendar::from_data(prop.clone(), x.href.clone())
    })
        .filter(|e| { e.resourcetype.collection.is_some() && e.resourcetype.calendar.is_some() })
        .collect::<Vec<CaldavCalendar>>())
}

/**
 * Get the events from the CalDAV server.
 */
pub async fn get_events(
    data: &CaldavCalendar,
    url: String,
    username: String,
    password: Option<String>
) -> Result<Vec<CaldavCalendarEvents>, anyhow::Error> {

    let method = reqwest::Method::from_bytes(b"REPORT").unwrap();
    let client = reqwest::Client::new();

    // Serialize the payload
    let payload = match to_xml_string(&CalendarQuery {
        xmlns_c: "urn:ietf:params:xml:ns:caldav".to_string(),
        xmlns_d: "DAV:".to_string(),
        prop: EventRequestProp::make(),
        filter: Filter {
            comp_filter: CompFilter {
                name: "VCALENDAR".to_string(),
                comp_filter: Some(Box::new(CompFilter {
                    name: "VEVENT".to_string(),
                    comp_filter: None,
                })),
            },
        },
    }) {
        Ok(payload) => {
            payload
        },
        Err(err) => {
            println!("Event Error {:?}", err);
            return Err(anyhow::anyhow!("get_events: Error serializing payload"));
        }
    };
    
    // Send the get events request
    let Ok(response) = client
        .request(method, url + &data.path)
        .header("Depth", "1")
        .basic_auth(username, password)
        .body(payload).send().await else {
        return Err(anyhow::anyhow!("get_events: Error sending request"));
    };

    // Expect a 207 status code
    if response.status() != 207 {
        return Err(anyhow::anyhow!("get_events: Error status code: {}", response.status()));
    }

    // Read the response
    let Ok(text) = response.text().await else {
        return Err(anyhow::anyhow!("get_events: Error reading response"));
    };

    // Deserialize the response
    let Ok(data) = parse_xml::<MultiStatus<EventResponse>>(&text) else {
        return Err(anyhow::anyhow!("get_events: Error deserializing response"));
    };

    // Filter responses with a 200 status code
    let elements = data.response.iter().filter(|x| {
        let Some(propstat) = x.propstat.first() else {
            return false;
        };
        propstat.status == "HTTP/1.1 200 OK"
    });

    // Loop through the elements and extract the etag and calendar data
    let mut list = elements.map(|events_data| {
        let propstat = events_data.propstat.first().unwrap();
        let prop = propstat.prop.as_ref().unwrap();

        let Some(etag) = &prop.getetag else {
            return None;
        };

        let Some(calendar_data) = &prop.calendar_data else {
            return None;
        };

        let cursor = Cursor::new(calendar_data.as_bytes());
        let buffered_reader = BufReader::new(cursor);
        let reader = ical::IcalParser::new(buffered_reader);

        let mut events: Vec<CaldavEvent> = Vec::new();

        for line in reader {
            match line {
                Ok(calendar) => {
                    calendar.events.iter().for_each(|event| {
                        events.push(CaldavEvent::from_ical_evel(event.clone()));
                    });
                },
                Err(_) => {
                    println!("Error");
                }
            }
        }
    
        Some(CaldavCalendarEvents {
            etag: etag.to_string(),
            events,
        })
    })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect::<Vec<CaldavCalendarEvents>>();

    if list.len() == 0 {
        return Err(anyhow::anyhow!("get_events: Error - list is empty"));
    }

    Ok(list)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "d:propfind")]
struct Propfind<T: Serialize> {
    #[serde(rename = "@xmlns:d")]
    d: String,
    #[serde(rename = "@xmlns:cal")]
    cal: String,
    #[serde(rename = "@xmlns:cs")]
    cs: String,
    #[serde(rename = "d:prop")]
    prop: T,
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PrincipalRequestProp {
    #[serde(rename = "d:current-user-principal")]
    current_user_principal: String,
}

impl PrincipalRequestProp {
    fn make() -> Self {
        PrincipalRequestProp {
            current_user_principal: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CalendarRequestProp {
    #[serde(rename = "@xmlns:d")]
    xmlns_d: String,
    #[serde(rename = "@xmlns:apple")]
    xmlns_apple: String,
    #[serde(rename = "@xmlns:ns")]
    xmlns_sc: String,
    #[serde(rename = "@xmlns:cal")]
    xmlns_cal: String,
    #[serde(rename = "d:current-user-privilege-set")]
    current_user_privilege_set: String,
    #[serde(rename = "d:displayname")]
    displayname: String,
    #[serde(rename = "d:description")]
    description: String,
    #[serde(rename = "d:resourcetype")]
    resource_type: String,
    #[serde(rename = "cs:source")]
    source: String,
    #[serde(rename = "apple:calendar-color")]
    calendar_color: String,
    #[serde(rename = "cal:supported-calendar-component-set")]
    supported_calendar_component_set: String,
    #[serde(rename = "cal:timezone")]
    timezone: String,
}

impl CalendarRequestProp {
    fn make() -> Self {
        CalendarRequestProp {
            xmlns_d: "DAV:".to_string(),
            xmlns_cal: "urn:ietf:params:xml:ns:caldav".to_string(),
            xmlns_sc: "http://calendarserver.org/ns/".to_string(),
            xmlns_apple: "http://apple.com/ns/ical/".to_string(),
            current_user_privilege_set: "".to_string(),
            displayname: "".to_string(),
            description: "".to_string(),
            resource_type: "".to_string(),
            calendar_color: "".to_string(),
            source: "".to_string(),
            supported_calendar_component_set: "".to_string(),
            timezone: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)] 
struct EventRequestProp {
    #[serde(rename = "d:getetag")]
    getetag: String,
    #[serde(rename = "c:calendar-data")]
    calendar_data: CalendarData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CalendarData {
    #[serde(rename = "c:comp")]
    comp: Comp
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Comp {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "$value")]
    children: Option<Vec<CompType>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum CompType {
    #[serde(rename = "c:comp")]
    Comp(Comp),
    #[serde(rename = "c:prop")]
    Prop(Prop)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "c:prop")]
struct Prop {
    #[serde(rename = "@name")]
    name: String,
}

impl EventRequestProp {
    fn make() -> Self {
        EventRequestProp {
            getetag: "".to_string(),
            calendar_data: CalendarData {
                comp: Comp {
                    name: "VCALENDAR".to_string(),
                    children: Some(vec![
                        CompType::Prop(Prop { name: "VERSION".to_string() }),
                        CompType::Comp(
                            Comp {
                                name: "VEVENT".to_string(),
                                children: Some(vec![
                                    CompType::Prop(Prop { name: "UID".to_string() }),
                                    CompType::Prop(Prop { name: "CREATED".to_string() }),
                                    CompType::Prop(Prop { name: "LAST-MODIFIED".to_string() }),
                                    CompType::Prop(Prop { name: "SUMMARY".to_string() }),
                                    CompType::Prop(Prop { name: "DTSTART".to_string() }),
                                    CompType::Prop(Prop { name: "DTEND".to_string() }),
                                    CompType::Prop(Prop { name: "ORGANIZER".to_string() }),
                                    CompType::Prop(Prop { name: "STATUS".to_string() }),
                                    CompType::Prop(Prop { name: "RECURRENCE-ID".to_string() }),
                                    CompType::Prop(Prop { name: "RRULE".to_string() }),
                                    CompType::Prop(Prop { name: "LOCATION".to_string() }),
                                    CompType::Prop(Prop { name: "TRANSP".to_string() }),
                                    CompType::Prop(Prop { name: "CATEGORIES".to_string() }),
                                    CompType::Prop(Prop { name: "ATTACH".to_string() }),
                                    CompType::Prop(Prop { name: "ATTENDEE".to_string() }),
                                ])
                            }
                        )
                    ]),
                },
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "c:calendar-query")]
struct CalendarQuery<T: Serialize> {
    #[serde(rename = "@xmlns:c")]
    xmlns_c: String,
    #[serde(rename = "@xmlns:d")]
    xmlns_d: String,
    #[serde(rename = "d:prop")]
    prop: T,
    #[serde(rename = "c:filter")]
    filter: Filter,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Filter {
    #[serde(rename = "c:comp-filter")]
    comp_filter: CompFilter,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CompFilter {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "c:comp-filter", skip_serializing_if = "Option::is_none")]
    comp_filter: Option<Box<CompFilter>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "multistatus")]
struct MultiStatus<T: Serialize> {
    response: Vec<PropfindResponse<T>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PropfindResponse<T: Serialize> {
    href: String,
    propstat: Vec<Propstat<T>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct EventResponse {
    #[serde(rename = "getetag", skip_serializing_if = "Option::is_none")]
    getetag: Option<String>,
    #[serde(rename = "calendar-data", default)]
    calendar_data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Propstat<T: Serialize> {
    prop: Option<T>,
    status: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CurrentPrincipleProp {
    #[serde(rename = "current-user-principal")]
    current_user_principal: Href,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Href {
    href: String,
}

#[derive(Debug)]
pub struct PrincipalData {
    pub user_id: String,
    pub path: String,
}

/**
 * Data structure for the principal data. Contains the user_id and the full path
 * to the principal.
 */
impl PrincipalData {
    fn new(user_id: String, path: String) -> Self {
        PrincipalData {
            user_id,
            path,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct CalendarPrivilege {}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct ResourceTypeCalendar {}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct ResourceTypeCollection {}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct CalendarResourceType {
    calendar: Option<ResourceTypeCalendar>,
    collection: Option<ResourceTypeCollection>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct CalendarComponent {
    // #[serde(rename = "name")]
    // name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct SupportCalendarComponentSet {
    #[serde(rename = "comp")]
    components: Option<Vec<CalendarComponent>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct CalendarResponseData {
    displayname: Option<String>,
    resourcetype: Option<CalendarResourceType>,
    #[serde(rename = "current-user-privilege-set")]
    privileges: Option<Vec<CalendarPrivilege>>,
    #[serde(rename = "calendar-color")]
    calendar_color: Option<String>,
    #[serde(rename = "supported-calendar-component-set")]
    components: Option<SupportCalendarComponentSet>,
}

#[derive(Debug)]
pub struct CaldavCalendar {
    pub id: String,
    pub path: String,
    pub displayname: String,
    pub resourcetype: CalendarResourceType,
    pub privileges: Vec<CalendarPrivilege>,
    pub calendar_color: String,
    pub components: SupportCalendarComponentSet,
}

impl CaldavCalendar {
    fn from_data(data: CalendarResponseData, path: String) -> Self {
        let id = path.trim_matches('/').split("/").last().unwrap();
        CaldavCalendar {
            id: id.to_string(),
            path: path,
            displayname: data.displayname.unwrap_or("".to_string()),
            resourcetype: data.resourcetype.unwrap_or(CalendarResourceType {
                calendar: None,
                collection: None,
            }),
            privileges: data.privileges.unwrap_or(Vec::new()),
            calendar_color: data.calendar_color.unwrap_or("".to_string()),
            components: data.components.unwrap_or(SupportCalendarComponentSet{
                components: None,
            })
        }
    }
}

#[derive(Debug)]
pub struct CaldavCalendarEvents {
    pub etag: String,
    pub events: Vec<CaldavEvent>,
}

/**
 * Represent the CalDAV event data structure
 */
#[derive(Debug)]
pub struct CaldavEvent {
    pub uid: String,
    pub created: String,
    pub last_modified: Option<String>,
    pub summary: String,
    pub dtstart: String,
    pub dtend: String,
    pub status: Option<String>,
    pub organizer: Option<String>,
    pub recurrence_id: Option<String>,
    pub rrule: Option<String>,
    pub location: Option<String>,
    pub transp: Option<String>,
    pub categories: Option<String>,
    pub attach: Option<String>,
    pub attendee: Option<String>,
}

/**
 * Convert an ical.rs IcalEvent to a CaldavEvent
 */
impl CaldavEvent {
    fn from_ical_evel(event: IcalEvent) -> Self {
        let mut property_map: HashMap<String, Property> = HashMap::new();
        for property in event.properties {
            property_map.insert(property.name.clone(), property);
        }

        fn get_value_safe(property_map: &HashMap<String, Property>, key: String) -> Option<String> {
            let Some(property) = property_map.get(&key) else {
                return None
            };
            
            let Some(value) = property.value.as_ref() else {
                return None
            };

            Some(value.clone())
        }

        Self {
            uid: get_value_safe(&property_map, "UID".to_string()).unwrap(),
            created: get_value_safe(&property_map, "CREATED".to_string()).unwrap(),
            summary: get_value_safe(&property_map, "SUMMARY".to_string()).unwrap(),
            last_modified: get_value_safe(&property_map, "LAST-MODIFIED".to_string()),
            dtstart: get_value_safe(&property_map, "DTSTART".to_string()).unwrap(),
            dtend: get_value_safe(&property_map, "DTEND".to_string()).unwrap(),
            status: get_value_safe(&property_map, "STATUS".to_string()),
            organizer: get_value_safe(&property_map, "ORGANIZER".to_string()),
            recurrence_id: get_value_safe(&property_map, "RECURRENCE-ID".to_string()),
            rrule: get_value_safe(&property_map, "RRULE".to_string()),
            location: get_value_safe(&property_map, "LOCATION".to_string()),
            transp: get_value_safe(&property_map, "TRANSP".to_string()),
            categories: get_value_safe(&property_map, "CATEGORIES".to_string()),
            attach: get_value_safe(&property_map, "ATTACH".to_string()),
            attendee: get_value_safe(&property_map, "ATTENDEE".to_string()),
        }
    }
}