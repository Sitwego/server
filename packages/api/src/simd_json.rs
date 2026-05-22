// use simd_json::base::ValueAsArray;
// use simd_json::base::ValueAsObject;
// use simd_json::base::ValueAsScalar;
// use simd_json::derived::ValueObjectAccess;
// use simd_json::from_slice;
// use simd_json::prelude::Indexed;
// use simd_json::OwnedValue;

// pub fn parse_from_string(json_str: &str) -> std::option::Option<(Vec<(f64, f64)>, f64, f64)> {
//   let mut json_bytes = json_str.as_bytes().to_vec();

//   let parsed_json: OwnedValue = from_slice(&mut json_bytes).ok()?;

//   let routes = parsed_json.get("routes")?.as_array()?;

//   let first_route = routes.get(0)?.as_object()?;

//   let geometry = first_route.get("geometry")?.as_object()?;

//   let coordinates = geometry.get("coordinates")?.as_array()?;

//   let extracted_coords: Vec<(f64, f64)> = coordinates
//       .iter()
//       .filter_map(|coord| {
//           let arr = coord.as_array()?;
//           let lon = arr.get(0)?.as_f64()?;
//           let lat = arr.get(1)?.as_f64()?;
//           Some((lon, lat))
//       })
//       .collect();

//   let distance = first_route.get("distance")?.as_f64()?;
//   let duration = first_route.get("duration")?.as_f64()?;
//   Some((extracted_coords, distance, duration))
// }

use simd_json::OwnedValue;
use simd_json::base::{ValueAsArray, ValueAsObject, ValueAsScalar};
use simd_json::derived::ValueObjectAccess;
use simd_json::from_slice;
use std::error::Error;

type EtaResult = (Vec<(f64, f64)>, f64, u64);

#[derive(Debug)]
pub enum ParseError {
    JsonParse(simd_json::Error),
    MissingField(&'static str),
    InvalidType(&'static str),
    InvalidCoordinateFormat,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::JsonParse(e) => write!(f, "JSON parsing error: {}", e),
            ParseError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ParseError::InvalidType(field) => {
                write!(f, "Invalid type for field: {}", field)
            }
            ParseError::InvalidCoordinateFormat => {
                write!(f, "Invalid coordinate format: must be [lon, lat]")
            }
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::JsonParse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<simd_json::Error> for ParseError {
    fn from(err: simd_json::Error) -> Self {
        ParseError::JsonParse(err)
    }
}

pub fn parse_from_string(json_str: &str) -> Result<EtaResult, ParseError> {
    let mut json_bytes = json_str.as_bytes().to_vec();

    let parsed_json: OwnedValue = from_slice(&mut json_bytes)?;

    // Navigate to routes array
    let routes = parsed_json
        .get("routes")
        .ok_or(ParseError::MissingField("routes"))?
        .as_array()
        .ok_or(ParseError::InvalidType("routes"))?;

    let first_route = routes
        .first() //get(0)
        .ok_or(ParseError::MissingField("routes[0]"))?
        .as_object()
        .ok_or(ParseError::InvalidType("routes[0]"))?;

    let geometry = first_route
        .get("geometry")
        .ok_or(ParseError::MissingField("geometry"))?
        .as_object()
        .ok_or(ParseError::InvalidType("geometry"))?;

    let coordinates = geometry
        .get("coordinates")
        .ok_or(ParseError::MissingField("coordinates"))?
        .as_array()
        .ok_or(ParseError::InvalidType("coordinates"))?;

    let extracted_coords: Vec<(f64, f64)> = coordinates
        .iter()
        .filter_map(|coord| {
            let arr = coord.as_array()?;
            if arr.len() != 2 {
                return None; // Silently skip invalid coordinates, or use ParseError if stricter
            }
            let lon = arr.first()?.as_f64()?;
            let lat = arr.get(1)?.as_f64()?;
            Some((lon, lat))
        })
        .collect();

    let distance = first_route
        .get("distance")
        .ok_or(ParseError::MissingField("distance"))?;

    let distance = match (distance.as_u64(), distance.as_f64()) {
        (Some(int_val), _) => int_val as f64,
        (None, Some(float_val)) => float_val.round(),
        _ => return Err(ParseError::InvalidType("distance")),
    };

    let duration_val = first_route
        .get("duration")
        .ok_or(ParseError::MissingField("duration"))?;

    let duration = match (duration_val.as_u64(), duration_val.as_f64()) {
        (Some(int_val), _) => int_val,
        (None, Some(float_val)) => float_val.round() as u64,
        _ => return Err(ParseError::InvalidType("duration")),
    };

    Ok((extracted_coords, distance, duration))
}
