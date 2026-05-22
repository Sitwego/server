use std::sync::Arc;

use db_store::Database;
use notif_api::{GorushClient, NotificationBuilder, Platform};
use sea_orm::EntityTrait;
use tracing::{error, warn};

use crate::schemas::profile;

/// Spawns a background task that fetches the profile by `profile_id`
///
/// The `configure` closure receives a [`NotificationBuilder`] already populated
/// with the device token and platform, and should chain any additional builder
/// methods (title, message, android config, priority, etc.) before returning.
///
/// Errors are logged as warnings; the calling handler is never blocked.
pub fn spawn_notify(
    db: Arc<Database>,
    notif: Arc<GorushClient>,
    profile_id: String,
    configure: impl FnOnce(NotificationBuilder) -> NotificationBuilder
    + Send
    + 'static,
) {
    tokio::spawn(async move {
        let profile_row =
            profile::Entity::find_by_id(&profile_id).one(db.conn()).await;

        match profile_row {
            Ok(Some(profile)) => {
                let token = match profile.device_token {
                    Some(ref t) => t.clone(),
                    None => {
                        warn!(
                            profile_id,
                            "no device_token on profile, skipping notification"
                        );
                        return;
                    }
                };

                let platform = match profile
                    .client_device_type
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .as_str()
                {
                    t if t.contains("ios") => Platform::Ios,
                    _ => Platform::Android,
                };

                let builder =
                    NotificationBuilder::new().token(token).platform(platform);

                match configure(builder).build() {
                    Ok(n) => {
                        if let Err(err) = notif.send(n).await {
                            error!(
                                ?err,
                                profile_id, "failed to send push notification"
                            );
                        }
                    }
                    Err(err) => {
                        error!(
                            profile_id,
                            err, "failed to build push notification"
                        );
                    }
                }
            }
            Ok(None) => {
                error!(profile_id, "profile not found, skipping notification");
            }
            Err(err) => {
                error!(
                    ?err,
                    profile_id, "db error fetching profile for notification"
                );
            }
        }
    });
}
