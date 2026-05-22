use serde::{Deserialize, Deserializer, Serialize};

/// How chatty a driver prefers to be during rides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chattiness {
    Chatty,
    Quiet,
    Considerate,
}

/// Music preference during rides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MusicPreference {
    Chill,
    Mood,
    Silent,
}

/// Smoking preference during rides.
///
/// Variants use explicit renames because `nosmoking` is not standard snake_case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SmokingPreference {
    #[serde(rename = "allowed")]
    Allowed,
    #[serde(rename = "considerate")]
    Considerate,
    #[serde(rename = "nosmoking")]
    NoSmoking,
}

/// Pet preference during rides.
///
/// Variants use explicit renames to match the frontend keys exactly
/// (`pets_allowed`, `no_pets`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PetPreference {
    #[serde(rename = "pets_allowed")]
    PetsAllowed,
    #[serde(rename = "considerate")]
    Considerate,
    #[serde(rename = "no_pets")]
    NoPets,
}

/// Travel preferences stored as a JSONB blob on the `profile` table.
///
/// All fields are `Option<_>` so the frontend can send partial updates.
/// Missing or `null` JSON fields deserialize to `None`; `None` fields are
/// omitted when serializing (the frontend infers "not set" from absence).
///
/// # Example JSON
/// ```json
/// {"chattiness":"considerate","music":"chill","smoking":"nosmoking","pets":"pets_allowed","max_ride_radius_km":5.0}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TravelPreferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chattiness: Option<Chattiness>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<MusicPreference>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smoking: Option<SmokingPreference>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pets: Option<PetPreference>,

    /// Maximum distance in km from the driver's current location to the ride
    /// pickup point. Offers beyond this radius are not sent to the driver.
    /// `None` means no restriction — the system default radius applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_ride_radius_km: Option<f64>,
}

impl TravelPreferences {
    /// Applies a partial update from the PUT request body.
    ///
    /// Each field in `update` is one of three states:
    /// - `None`        → field was absent in JSON — leave existing value unchanged
    /// - `Some(None)`  → field was explicitly `null` — clear the stored value
    /// - `Some(Some(v))` → field had a value — overwrite with `v`
    pub fn merge(&mut self, update: TravelPreferencesUpdate) {
        if let Some(v) = update.chattiness {
            self.chattiness = v;
        }
        if let Some(v) = update.music {
            self.music = v;
        }
        if let Some(v) = update.smoking {
            self.smoking = v;
        }
        if let Some(v) = update.pets {
            self.pets = v;
        }
        if let Some(v) = update.max_ride_radius_km {
            self.max_ride_radius_km = v;
        }
    }

    /// Serializes to a `serde_json::Value` for storage in the DB.
    pub fn to_json_value(
        &self,
    ) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Deserializes from a `serde_json::Value` read from the DB.
    pub fn from_json_value(
        value: &serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }
}

/// Wraps `Option::deserialize` in `Some` so we can distinguish:
/// - field absent in JSON → outer `None` (via `#[serde(default)]`)
/// - field present as `null` → `Some(None)`
/// - field present with a value → `Some(Some(v))`
fn deserialize_optional_field<'de, T, D>(
    d: D,
) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(d)?))
}

/// Request body for `PUT /api/driver/{driver_id}/preferences`.
///
/// Each field is a tri-state:
/// - absent in JSON  → `None`         → leave existing value unchanged
/// - `null` in JSON  → `Some(None)`   → clear the stored value
/// - a string value  → `Some(Some(v))` → overwrite with the new value
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TravelPreferencesUpdate {
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub chattiness: Option<Option<Chattiness>>,

    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub music: Option<Option<MusicPreference>>,

    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub smoking: Option<Option<SmokingPreference>>,

    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub pets: Option<Option<PetPreference>>,

    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub max_ride_radius_km: Option<Option<f64>>,
}
