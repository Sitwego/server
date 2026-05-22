use std::path::Path;
use std::time::Duration;

use db_store::*;
use utils::collections::HashMap;

pub async fn run_db_migrations(
    migration_sourse: impl AsRef<Path>,
    db_conn_opt: &ConnectOptions,
) -> anyhow::Result<Vec<(Migration, Duration)>> {
    let migration_path = migration_sourse.as_ref();

    let migrations =
        MigrationSource::resolve(migration_path).await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to load migrations from {:?}: {e:?}",
                migration_path
            )
        })?;

    let mut db_conn = AnyConnection::connect(db_conn_opt.get_url()).await?;

    db_conn.ensure_migrations_table().await?;
    let migrations_applied: HashMap<_, _> = db_conn
        .list_applied_migrations()
        .await?
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    let mut new_migrations = Vec::new();
    for migration in migrations {
        match migrations_applied.get(&migration.version) {
            Some(applied_migration) => {
                if migration.checksum != applied_migration.checksum {
                    Err(anyhow::anyhow!(
                        "Checksum mismatch for applied migration {}",
                        migration.description
                    ))?;
                }
            }
            None => {
                let elapsed = db_conn.apply(&migration).await?;
                new_migrations.push((migration, elapsed));
            }
        }
    }

    db_conn.close().await?;

    Ok(new_migrations)
}
