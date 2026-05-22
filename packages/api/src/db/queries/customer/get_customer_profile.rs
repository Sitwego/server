use async_trait::async_trait;
use db_store::Database;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use utils::Result;

/// Raw result fetched from the DB — contact_data/nonce still encrypted.
/// The API layer decrypts these to get email + phone_number.
#[derive(Debug)]
pub struct RiderProfileRow {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub contact_data: Vec<u8>,
    pub nonce: Vec<u8>,
    /// KMS ciphertext blob — passed to extract_contact_info to decrypt contact_data.
    pub encrypted_key: Vec<u8>,
    pub email_verified: bool,
    pub face_image_id: Option<String>,
    pub mobile_country_code: Option<String>,
    pub dob: Option<Date>,
    pub google_linked: bool,
    pub google_email: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    // rider_stats (LEFT JOIN — NULL when no stats row yet)
    pub rating: Option<f64>,
    pub total_ratings: Option<i32>,
    pub total_rating_score: Option<f64>,
    // profile_address (LEFT JOIN — NULL when no address set)
    pub addr_street: Option<String>,
    pub addr_city: Option<String>,
    pub addr_state: Option<String>,
    pub addr_zip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressData {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}

/// The fully-assembled rider profile returned to the client.
/// Build this in the API handler after decrypting contact_data.
#[derive(Debug, Serialize, Deserialize)]
pub struct RiderProfile {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub email_verified: bool,
    pub phone_number: String,
    pub mobile_country_code: String,
    pub age: Option<i32>,
    pub avatar_url: Option<String>,
    pub rating: f64,
    pub total_rating_score: f64,
    pub review_count: i32,
    pub google_linked: bool,
    pub google_email: Option<String>,
    pub address: Option<AddressData>,
    pub created_at: String,
    pub updated_at: String,
}

impl RiderProfileRow {
    /// Convert the raw DB row into the response shape.
    /// `email` and `phone_number` come from decrypting `contact_data`.
    pub fn into_profile(
        self,
        email: String,
        phone_number: String,
    ) -> RiderProfile {
        let age = self.dob.map(calculate_age);
        let address = if self.addr_street.is_some()
            || self.addr_city.is_some()
            || self.addr_state.is_some()
            || self.addr_zip.is_some()
        {
            Some(AddressData {
                street: self.addr_street,
                city: self.addr_city,
                state: self.addr_state,
                zip: self.addr_zip,
            })
        } else {
            None
        };

        RiderProfile {
            id: self.id,
            first_name: self.first_name,
            last_name: self.last_name,
            email,
            email_verified: self.email_verified,
            phone_number,
            mobile_country_code: self
                .mobile_country_code
                .unwrap_or_else(|| "+254".to_string()),
            age,
            avatar_url: self.face_image_id,
            rating: self.rating.unwrap_or(0.0),
            total_rating_score: self.total_rating_score.unwrap_or(0.0),
            review_count: self.total_ratings.unwrap_or(0),
            google_linked: self.google_linked,
            google_email: self.google_email,
            address,
            created_at: self.created_at.to_string(),
            updated_at: self.updated_at.to_string(),
        }
    }
}

fn calculate_age(dob: Date) -> i32 {
    let today = OffsetDateTime::now_utc().date();
    let mut age = today.year() - dob.year();
    if today.month() < dob.month()
        || (today.month() == dob.month() && today.day() < dob.day())
    {
        age -= 1;
    }
    age
}

fn tg<T: sea_orm::TryGetable>(
    row: &sea_orm::QueryResult,
    col: &str,
) -> std::result::Result<T, DbErr> {
    row.try_get("", col).map_err(|e| DbErr::Custom(e.to_string()))
}

#[async_trait]
pub trait GetRiderProfile {
    async fn get_rider_profile(
        &self,
        rider_id: &str,
    ) -> Result<Option<RiderProfileRow>>;

    // get rider small profile for dispatching (id, name, rating, contact)
    async fn get_rider_small_profile(
        &self,
        rider_id: &str,
    ) -> Result<Option<RiderSmallProfileRow>>;
}

#[derive(Debug)]
pub struct RiderSmallProfileRow {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub contact_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
    pub mobile_country_code: Option<String>,
    pub rating: Option<f64>,
    pub total_rating_score: Option<f64>,
}

#[async_trait]
impl GetRiderProfile for Database {
    async fn get_rider_profile(
        &self,
        rider_id: &str,
    ) -> Result<Option<RiderProfileRow>> {
        self.transaction(move |tx| {
            let rider_id = rider_id.to_string();
            async move {
                let stmt = Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                        SELECT
                            p.id,
                            p.first_name,
                            p.last_name,
                            p.contact_data,
                            p.nonce,
                            p.encrypted_key,
                            p.verified              AS email_verified,
                            p.face_image_id,
                            p.mobile_country_code,
                            p.dob,
                            p.google_linked,
                            p.google_email,
                            p.created_at,
                            p.updated_at,
                            CAST(rs.rating AS FLOAT8) AS rating,
                            rs.total_ratings,
                            rs.total_rating_score,
                            pa.street               AS addr_street,
                            pa.city                 AS addr_city,
                            pa.state                AS addr_state,
                            pa.zip                  AS addr_zip
                        FROM profile p
                        INNER JOIN customer       c  ON c.id           = p.id
                        LEFT  JOIN rider_stats    rs ON rs.customer_id  = p.id
                        LEFT  JOIN profile_address pa ON pa.profile_id  = p.id
                        WHERE p.id = $1
                    "#,
                    [rider_id.into()],
                );

                let row = match tx.query_one(stmt).await? {
                    None => return Ok(None),
                    Some(r) => r,
                };

                Ok(Some(RiderProfileRow {
                    id: tg(&row, "id")?,
                    first_name: tg(&row, "first_name")?,
                    last_name: tg(&row, "last_name")?,
                    contact_data: tg(&row, "contact_data")?,
                    nonce: tg(&row, "nonce")?,
                    encrypted_key: tg(&row, "encrypted_key")?,
                    email_verified: tg(&row, "email_verified")?,
                    face_image_id: tg(&row, "face_image_id")?,
                    mobile_country_code: tg(&row, "mobile_country_code")?,
                    dob: tg(&row, "dob")?,
                    google_linked: tg(&row, "google_linked")?,
                    google_email: tg(&row, "google_email")?,
                    created_at: tg(&row, "created_at")?,
                    updated_at: tg(&row, "updated_at")?,
                    rating: tg(&row, "rating")?,
                    total_ratings: tg(&row, "total_ratings")?,
                    total_rating_score: tg(&row, "total_rating_score")?,
                    addr_street: tg(&row, "addr_street")?,
                    addr_city: tg(&row, "addr_city")?,
                    addr_state: tg(&row, "addr_state")?,
                    addr_zip: tg(&row, "addr_zip")?,
                }))
            }
        })
        .await
    }

    async fn get_rider_small_profile(
        &self,
        rider_id: &str,
    ) -> Result<Option<RiderSmallProfileRow>> {
        self.transaction(move |tx| {
            let rider_id = rider_id.to_string();
            async move {
                let stmt = Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                        SELECT
                            p.id,
                            p.first_name,
                            p.last_name,
                            p.contact_data,
                            p.nonce,
                            p.encrypted_key,
                            p.mobile_country_code,
                            CAST(rs.rating AS FLOAT8) AS rating,
                            rs.total_rating_score
                        FROM profile p
                        INNER JOIN customer    c  ON c.id          = p.id
                        LEFT  JOIN rider_stats rs ON rs.customer_id = p.id
                        WHERE p.id = $1
                    "#,
                    [rider_id.into()],
                );

                let row = match tx.query_one(stmt).await? {
                    None => return Ok(None),
                    Some(r) => r,
                };
                Ok(Some(RiderSmallProfileRow {
                    id: tg(&row, "id")?,
                    first_name: tg(&row, "first_name")?,
                    last_name: tg(&row, "last_name")?,
                    contact_data: tg(&row, "contact_data")?,
                    nonce: tg(&row, "nonce")?,
                    encrypted_key: tg(&row, "encrypted_key")?,
                    mobile_country_code: tg(&row, "mobile_country_code")?,
                    rating: tg(&row, "rating")?,
                    total_rating_score: tg(&row, "total_rating_score")?,
                }))
            }
        })
        .await
    }
}
