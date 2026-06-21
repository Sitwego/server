use anyhow::Context;
use chrono::Utc;
use db_store::Database;
use sea_orm::sea_query::OnConflict;
use std::pin::Pin;

use crate::schemas::ride_request::RideRequestStatus;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait,
    DbBackend, EntityTrait, FromQueryResult, QueryFilter, Statement, Value,
};
use tokio::try_join;
use tracing::info_span;
// use time::OffsetDateTime;
use crate::schemas::{location, ride_request};
use crate::types::*;
use crate::{schemas::ride, types::DriverId};
// use tracing::{info, warn};

/// One exploded point of a ride's stored linestring (PostGIS X = lon, Y = lat).
#[derive(Debug, FromQueryResult)]
struct RideLineStringPoint {
    lat: f64,
    lon: f64,
}

type RideData = Result<
    Option<(
        ride_request::Model,
        Option<location::Model>,
        Option<location::Model>,
    )>,
    AppError,
>;

pub trait RideQueries {
    fn ride_exist(
        &self,
        ride_id: &RideId,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<ride::Model>, AppError>,
    > + Send;
    fn process_ride_coordinates(
        &self,
        driver_id: &DriverId,
        ride_id: &RideId,
        geopoints: &[GeoPoint],
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
    /// Fetch the recorded route for a ride as an ordered list of points.
    /// Returns an empty vec when the ride has no recorded coordinates.
    fn get_ride_line_string(
        &self,
        ride_id: &RideId,
    ) -> impl std::future::Future<Output = utils::Result<Vec<GeoPoint>>> + Send;
    fn create_locations(
        &self,
        from: location::ActiveModel,
        to: location::ActiveModel,
    ) -> impl std::future::Future<Output = utils::Result<(String, String)>> + Send;
    fn create_ride_request(
        &self,
        rq_model: ride_request::Model,
        from: String,
        to: String,
        end_otp: String,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
    fn update_ride_request_status(
        &self,
        ride_id: RideId,
        new_status: RideRequestStatus,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
    fn get_current_ride_info(
        &self,
        ride_id: RideId,
    ) -> impl std::future::Future<Output = utils::Result<RideData>> + Send;
    #[allow(clippy::too_many_arguments)]
    fn start_ride(
        &self,
        driver_id: &DriverId,
        ride_id: &RideId,
        start_otp: String,
        currency: String,
        a: GeoPoint,
        b: GeoPoint,
        is_ride_during_free_trial: bool,
        vc: &str,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;

    fn get_ride_request_by_id(
        &self,
        ride_request_id: &RideId,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<ride_request::Model>, AppError>,
    > + Send;

    fn get_ride_request_by_id_with_location(
        &self,
        ride_request_id: &RideId,
    ) -> impl std::future::Future<
        Output = utils::Result<
            Option<(ride_request::Model, Option<location::Model>)>,
            AppError,
        >,
    > + Send;

    fn get_completed_ride_by_id(
        &self,
        ride_id: &RideId,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<ride::Model>, AppError>,
    > + Send;

    //Delete ride request by id
    fn delete_ride_request_by_id(
        &self,
        ride_request_id: &RideId,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
}

impl RideQueries for Database {
    async fn ride_exist(
        &self,
        ride_id: &RideId,
    ) -> utils::Result<Option<ride::Model>, AppError> {
        let ride = self
            .transaction(move |tx| {
                Box::pin(async move {
                    let ride_id = &ride_id.0;
                    let ride =
                        ride::Entity::find_by_id(ride_id).one(&*tx).await?;
                    Ok(ride)
                })
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(ride)
    }
    async fn process_ride_coordinates(
        &self,
        driver_id: &DriverId,
        ride_id: &RideId,
        geopoints: &[GeoPoint],
    ) -> utils::Result<()> {
        // PostGIS WKT uses "X Y" = "longitude latitude" ordering.
        let line_string = geopoints
            .iter()
            .map(|point| format!("{} {}", point.lon.0, point.lat.0))
            .collect::<Vec<_>>()
            .join(",");
        let wkt = format!("LINESTRING({line_string})");

        info_span!("ride wkt", wkt = %wkt).in_scope(|| {
            tracing::debug!("Appending ride coordinates");
        });

        // Insert a fresh linestring, or append the new points onto the existing
        // geometry, in a single atomic statement. ST_MakeLine concatenates the
        // existing points (first) with the incoming ones, so we avoid the
        // app-side read-modify-write and the lost-update race it caused under
        // concurrent location pings. A missing ride is rejected by the
        // ride_history -> ride foreign key, so no separate existence check.
        let sql_stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO ride_history (ride_id, driver_id, coordinates, updated_at) \
             VALUES ($1, $2, ST_GeogFromText($3), NOW()) \
             ON CONFLICT (ride_id) DO UPDATE SET \
             coordinates = ST_MakeLine(\
                 ride_history.coordinates::geometry, EXCLUDED.coordinates::geometry\
             )::geography, \
             updated_at = NOW()",
            vec![
                Value::String(Some(Box::new(ride_id.0.clone()))),
                Value::String(Some(Box::new(driver_id.0.clone()))),
                Value::String(Some(Box::new(wkt))),
            ],
        );

        self.transaction(move |tx| {
            let value = sql_stmt.clone();
            async move {
                let re: sea_orm::ExecResult = tx
                    .execute(value)
                    .await
                    .context("Failed to upsert ride_history coordinates")?;
                if re.rows_affected() == 0 {
                    return Err(
                        anyhow::anyhow!("Failed to write ride history").into()
                    );
                }
                Ok(())
            }
        })
        .await
        .context("Transaction failed")?;

        Ok(())
    }

    async fn get_ride_line_string(
        &self,
        ride_id: &RideId,
    ) -> utils::Result<Vec<GeoPoint>> {
        // Explode the stored geography into ordered points and read each
        // coordinate directly via ST_X (longitude) / ST_Y (latitude), so we
        // never have to parse WKT text on the Rust side.
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT ST_Y(dp.geom) AS lat, ST_X(dp.geom) AS lon \
             FROM ride_history rh, \
                  LATERAL ST_DumpPoints(rh.coordinates::geometry) AS dp \
             WHERE rh.ride_id = $1 \
             ORDER BY dp.path",
            vec![Value::String(Some(Box::new(ride_id.0.clone())))],
        );

        let rows = RideLineStringPoint::find_by_statement(stmt)
            .all(self.conn())
            .await
            .context("Failed to query ride line string")?;

        Ok(rows
            .into_iter()
            .map(|p| GeoPoint {
                lat: Latitude(p.lat),
                lon: Longitude(p.lon),
            })
            .collect())
    }

    async fn create_locations(
        &self,
        from_location: location::ActiveModel,
        to_location: location::ActiveModel,
    ) -> utils::Result<(String, String)> {
        self.transaction(move |tx| {
            let from = from_location.clone();
            let to = to_location.clone();
            async move {
                let from = async {
                    let id = location::Entity::insert(from)
                        .exec(&*tx)
                        .await?
                        .last_insert_id;
                    Ok::<String, utils::Error>(id)
                };
                let to = async {
                    let id = location::Entity::insert(to)
                        .exec(&*tx)
                        .await?
                        .last_insert_id;
                    Ok::<String, utils::Error>(id)
                };

                let (from, to) = try_join!(from, to)?;

                Ok((from, to))
            }
        })
        .await
    }

    async fn create_ride_request(
        &self,
        updated_rq_model: ride_request::Model,
        from_id: String,
        to_id: String,
        end_otp: String,
    ) -> utils::Result<()> {
        self.transaction(move |tx| {
            let id = updated_rq_model.id.clone();
            let driver_id = updated_rq_model.driver_id.clone();
            let customer_id = updated_rq_model.customer_id.clone();
            let rq_status = updated_rq_model.request_status;
            let from_location_id = from_id.clone();
            let to_location_id = to_id.clone();
            let end_otp = end_otp.clone();
            let start_otp = updated_rq_model.otp.clone();
            Box::pin(async move {
                let rq_model = ride_request::ActiveModel {
                    id: ActiveValue::Set(id),
                    driver_id: ActiveValue::Set(driver_id),
                    customer_id: ActiveValue::Set(customer_id),
                    fare: ActiveValue::Set(updated_rq_model.fare),
                    request_status: ActiveValue::Set(rq_status),
                    estimated_distance_to_pickup: ActiveValue::Set(
                        updated_rq_model.estimated_distance_to_pickup,
                    ),
                    estimated_duration_to_pickup: ActiveValue::Set(
                        updated_rq_model.estimated_duration_to_pickup,
                    ),
                    estimated_distance: ActiveValue::Set(
                        updated_rq_model.estimated_distance,
                    ),
                    estimated_duration: ActiveValue::Set(
                        updated_rq_model.estimated_duration,
                    ),
                    search_request_valid_till: ActiveValue::Set(
                        updated_rq_model.search_request_valid_till,
                    ),
                    start_time: ActiveValue::Set(updated_rq_model.start_time),
                    from_location_id: ActiveValue::Set(from_location_id),
                    to_location_id: ActiveValue::Set(to_location_id),
                    created_at: ActiveValue::Set(Utc::now()),
                    updated_at: ActiveValue::Set(Utc::now()),
                    message: ActiveValue::Set(Some("".to_string())),
                    otp: ActiveValue::Set(start_otp),
                    end_otp: ActiveValue::Set(Some(end_otp.to_owned())),
                    otp_verified: ActiveValue::Set(false),
                    end_otp_verified: ActiveValue::Set(false),
                };
                ride_request::Entity::insert(rq_model)
                    .on_conflict(
                        OnConflict::column(ride_request::Column::Id)
                            .update_columns([
                                ride_request::Column::DriverId,
                                ride_request::Column::Fare,
                                ride_request::Column::RequestStatus,
                                ride_request::Column::EstimatedDistanceToPickup,
                                ride_request::Column::EstimatedDurationToPickup,
                                ride_request::Column::SearchRequestValidTill,
                                ride_request::Column::StartTime,
                                ride_request::Column::FromLocationId,
                                ride_request::Column::ToLocationId,
                                ride_request::Column::UpdatedAt,
                                ride_request::Column::Message,
                                ride_request::Column::Otp,
                            ])
                            .to_owned(),
                    )
                    .exec(&*tx)
                    .await?;
                Ok(())
            })
                as Pin<Box<dyn Future<Output = utils::Result<()>> + Send>>
        })
        .await?;
        Ok(())
    }

    async fn start_ride(
        &self,
        DriverId(driver_id): &DriverId,
        RideId(ride_id): &RideId,
        otp: String,
        c: String,           // currency
        srt_point: GeoPoint, // current_driver_location
        end_point: GeoPoint,
        is_ride_during_free_trial: bool,
        vc: &str, // vehicle category
    ) -> utils::Result<()> {
        let GeoPoint {
            lat: Latitude(srt_lat),
            lon: Longitude(srt_lng),
        } = srt_point;
        let GeoPoint {
            lat: Latitude(end_lat),
            lon: Longitude(end_lng),
        } = end_point;
        let _ride_mdl = self
            .transaction(move |tx| {
                let driver_id = driver_id.clone();
                let ride_id = ride_id.clone();
                let otp = otp.clone();
                let c = c.clone();
                async move {
                    let ride_request: Option<ride_request::Model> =
                        ride_request::Entity::find_by_id(ride_id)
                            .one(&*tx)
                            .await
                            .context("Ride does not exist.")?;
                    if let Some(ride_request) = ride_request.as_ref() {
                        let ride_request = ride_request.to_owned();
                        ride::ActiveModel {
                            id: ActiveValue::Set(ride_request.id),
                            customer_id: ActiveValue::Set(
                                ride_request.customer_id,
                            ),
                            status: ActiveValue::Set(
                                RideRequestStatus::Inprogress.to_string(),
                            ),
                            driver_id: ActiveValue::Set(driver_id),
                            otp: ActiveValue::Set(otp),
                            end_otp: ActiveValue::Set(None),
                            tracking_url: ActiveValue::Set("".to_string()),
                            fare: ActiveValue::Set(ride_request.fare.into()),
                            currency: ActiveValue::Set(Some(c)),
                            traveled_distance: ActiveValue::Set(0.0),
                            chargeable_distance: ActiveValue::Set(Some(0)),
                            driver_arrival_time: ActiveValue::Set(Some(
                                Utc::now().into(),
                            )), // TODO
                            trip_start_time: ActiveValue::Set(
                                Utc::now().into(),
                            ),
                            trip_start_lat: ActiveValue::Set(Some(srt_lat)),
                            trip_start_lon: ActiveValue::Set(Some(srt_lng)),
                            trip_end_lat: ActiveValue::Set(Some(end_lat)),
                            trip_end_lon: ActiveValue::Set(Some(end_lng)),
                            created_at: ActiveValue::Set(Utc::now().into()),
                            updated_at: ActiveValue::Set(Utc::now().into()),
                            safety_alert_triggered: ActiveValue::Set(false),
                            enable_frequent_location_updates: ActiveValue::Set(
                                Some(true),
                            ),
                            online_payment: ActiveValue::Set(false),
                            // distance_unit: todo!(),
                            // trip_end_time: todo!(),
                            // pickup_drop_outside_of_threshold: todo!(),
                            // driver_deviated_to_toll_route: todo!(),
                            // driver_deviated_from_route: todo!(),
                            // number_of_snap_to_road_calls: todo!(),
                            // number_of_osrm_snap_to_road_calls: todo!(),
                            // number_of_self_tuned: todo!(),
                            // number_of_deviation: todo!(),
                            // ui_distance_calculation_with_accuracy: todo!(),
                            // ui_distance_calculation_without_accuracy: todo!(),
                            // is_free_ride: todo!(),
                            // estimated_toll_charges: todo!(),
                            // estimated_toll_names: todo!(),
                            // ride_ended_by: todo!(),
                            trip_category: sea_orm::Set(Some(vc.to_string())),
                            // cancellation_fee_if_cancelled: todo!(),
                            // tip_amount: todo!(),
                            // ride_tags: todo!(),
                            // ride_type: todo!(),
                            is_ride_during_free_trial: ActiveValue::Set(
                                is_ride_during_free_trial,
                            ),
                            ..Default::default()
                        }
                        .insert(&*tx)
                        .await?;
                        return Ok(true);
                    }
                    Ok(false)
                }
            })
            .await?;
        Ok(())
    }

    async fn update_ride_request_status(
        &self,
        ride_id: RideId,
        new_status: RideRequestStatus,
    ) -> utils::Result<()> {
        self.transaction(move |tx| {
            let ride_id = ride_id.0.clone();
            let new_status = new_status;
            Box::pin(async move {
                let sql_stmt = Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE ride_requests SET request_status = $1::ride_request_status, updated_at = $2 WHERE id = $3",
                    vec![
                        Value::String(Some(Box::new(new_status.to_string()))),
                        Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
                        Value::String(Some(Box::new(ride_id.to_owned()))),
                    ],
                );
                //update the ride_request status
                let _: sea_orm::ExecResult = tx
                    .execute(sql_stmt)
                    .await
                    .map_err(| err | {
                        tracing::error!("Error updating ride_request table: {:?}", err);
                        err
                    })?;

                //first check if ride exist in the ride table
                let ride = ride::Entity::find_by_id(ride_id.clone())
                    .one(&*tx)
                    .await
                    .context("Ride does not exist")?;       
                if ride.is_none() {
                    return Ok(());
                }
                // update the ride status if ride exist
                let sql_stmt = Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE ride SET status = $1::ride_request_status, updated_at = $2, trip_end_time = $3 WHERE id = $4",
                    vec![
                        Value::String(Some(Box::new(new_status.to_string()))),
                        Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
                        Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
                        Value::String(Some(Box::new(ride_id))),
                    ],
                );
                let _: sea_orm::ExecResult = tx
                    .execute(sql_stmt)
                    .await
                    .map_err(| err | {
                        tracing::error!("Error updating ride table: {:?}", err);
                        err
                    })?;

                Ok(())
            })
        })
        .await
    }

    async fn get_current_ride_info(
        &self,
        ride_id: RideId,
    ) -> utils::Result<RideData> {
        self.transaction(move |tx| {
            let ride_id = ride_id.0.clone();
            async move {
                let ride = ride_request::Entity::find_by_id(ride_id)
                    .filter(
                        Condition::all().add(
                            ride_request::Column::RequestStatus
                                .eq(RideRequestStatus::Accepted),
                        ),
                    )
                    .one(&*tx)
                    .await
                    .context("Ride does not exist")?;
                let ride_data = if let Some(ride) = ride {
                    let (from_location, to_location) = tokio::try_join!(
                        location::Entity::find_by_id(
                            ride.from_location_id.clone()
                        )
                        .one(&*tx),
                        location::Entity::find_by_id(
                            ride.to_location_id.clone()
                        )
                        .one(&*tx),
                    )?;

                    let result = Some((ride, from_location, to_location));
                    Ok::<_, AppError>(result)
                } else {
                    Ok(None)
                };

                Ok(ride_data)
            }
        })
        .await
    }

    async fn get_ride_request_by_id(
        &self,
        ride_request_id: &RideId,
    ) -> utils::Result<Option<ride_request::Model>, AppError> {
        let ride_request = self
            .transaction(move |tx| {
                let ride_request_id = ride_request_id.0.clone();
                async move {
                    let ride_request =
                        ride_request::Entity::find_by_id(ride_request_id)
                            .one(&*tx)
                            .await
                            .context("Failed to fetch ride request by ID")?;
                    Ok(ride_request)
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(ride_request)
    }

    async fn get_ride_request_by_id_with_location(
        &self,
        ride_request_id: &RideId,
    ) -> utils::Result<
        Option<(ride_request::Model, Option<location::Model>)>,
        AppError,
    > {
        let ride_request_with_location = self
            .transaction(move |tx| {
                let ride_request_id = ride_request_id.0.clone();
                async move {
                    let ride_request_with_location = ride_request::Entity::find_by_id(ride_request_id)
                        .find_also_related(location::Entity)
                        .one(&*tx)
                        .await
                        .context("Failed to fetch ride request with location by ID")?;
                    Ok(ride_request_with_location)
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(ride_request_with_location)
    }

    async fn get_completed_ride_by_id(
        &self,
        ride_id: &RideId,
    ) -> utils::Result<Option<ride::Model>, AppError> {
        let ride = self
            .transaction(move |tx| {
                let ride_id = ride_id.0.to_string();
                async move {
                    let ride = ride::Entity::find_by_id(ride_id)
                        .filter(
                            Condition::all().add(
                                ride::Column::Status
                                    .eq(RideRequestStatus::Completed),
                            ),
                        )
                        .one(&*tx)
                        .await
                        .context("Failed to fetch completed ride by ID")?;
                    Ok(ride)
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        Ok(ride)
    }

    async fn delete_ride_request_by_id(
        &self,
        ride_request_id: &RideId,
    ) -> utils::Result<()> {
        self.transaction(move |tx| {
            let ride_request_id = ride_request_id.0.clone();
            async move {
                let res = ride_request::Entity::delete_by_id(ride_request_id)
                    .exec(&*tx)
                    .await
                    .context("Failed to delete ride request by ID")?;
                if res.rows_affected == 0 {
                    return Err(anyhow::anyhow!(
                        "No ride request found with the given ID"
                    )
                    .into());
                }
                Ok(())
            }
        })
        .await
    }
}
