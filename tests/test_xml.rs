use quick_xml::de::from_str;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    #[serde(rename = "@name")] // `name` is an XML attribute
    name: String,
    
    #[serde(rename = "@age")] // `age` is an XML attribute
    age: u32,
}

#[test]
fn test_xml_deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"<Person name="John Doe" age="30" />"#;

    // Deserialize the XML string into a Person struct
    let person: Person = from_str(xml)?;

    // Output the resulting struct
    println!("{:?}", person);

    Ok(())
}