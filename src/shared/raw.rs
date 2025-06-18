use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor, SeqAccess};
use std::fmt;

// ====================================================================
// Data structures for the starmap pickle extracted from the client
// ====================================================================

#[derive(Debug, Deserialize)]
pub struct RawStarMap {
    pub jumps: Vec<RawJump>,
    #[serde(rename(deserialize = "solarSystems"))]
    pub solar_systems: HashMap<String, RawSolarSystem>,
}

impl RawStarMap {
    pub fn from_file(file: &str) -> Self {
        let file = std::fs::read_to_string(file).unwrap();
        serde_json::from_str(&file).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct RawJump {
    #[serde(rename(deserialize = "fromSystemID"))]
    pub from_system_id: u32,
    #[serde(rename(deserialize = "jumpType"))]
    pub jump_type: u8,
    #[serde(rename(deserialize = "toSystemID"))]
    pub to_system_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct RawSolarSystem {
    #[serde(deserialize_with = "deserialize_center")]
    pub center: [f64; 3],
}

fn deserialize_center<'de, D>(deserializer: D) -> Result<[f64; 3], D::Error>
where
    D: Deserializer<'de>,
{
    struct CenterVisitor;

    impl<'de> Visitor<'de> for CenterVisitor {
        type Value = [f64; 3];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array of 3 numbers or strings that can be parsed as numbers")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = [0.0; 3];
            for i in 0..3 {
                let value = seq.next_element::<serde_json::Value>()?;
                match value {
                    Some(serde_json::Value::Number(n)) => {
                        result[i] = n.as_f64().ok_or_else(|| de::Error::custom("invalid number"))?;
                    }
                    Some(serde_json::Value::String(s)) => {
                        result[i] = s.parse::<f64>().map_err(de::Error::custom)?;
                    }
                    _ => return Err(de::Error::custom("expected number or string")),
                }
            }
            Ok(result)
        }
    }

    deserializer.deserialize_seq(CenterVisitor)
}

#[derive(Debug, Deserialize)]
pub struct RawStar {
    #[serde(rename(deserialize = "solarSystemId"))]
    pub solar_system_id: u64,
    #[serde(rename(deserialize = "solarSystemName"))]
    pub solar_system_name: String,
}