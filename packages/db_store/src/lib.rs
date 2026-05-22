use rand::{Rng, SeedableRng, prelude::StdRng};
pub use sea_orm::ConnectOptions;
use sea_orm::{
    DatabaseConnection, DatabaseTransaction, DbErr, IsolationLevel,
    TransactionTrait, entity::prelude::*,
};
pub use sqlx::{
    AnyConnection, Connection,
    migrate::{Migrate, Migration, MigrationSource},
};
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::warn;
use utils::*;
pub struct Database {
    options: ConnectOptions,
    pool: DatabaseConnection,
    rng: Mutex<StdRng>,
    executor: executor::Executor,
}

impl Database {
    pub async fn new(
        opt: ConnectOptions,
        executor: executor::Executor,
    ) -> Result<Self> {
        sqlx::any::install_default_drivers();
        Ok(Self {
            pool: sea_orm::Database::connect(opt.clone()).await?, // Connects to the database
            options: opt.clone(), // Store the ConnectOptions (cloned if necessary elsewhere)
            executor,
            rng: Mutex::new(StdRng::seed_from_u64(0)),
        })
    }

    pub fn options(&self) -> &ConnectOptions {
        &self.options
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.pool
    }

    pub async fn transaction<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Send + Fn(TransactionHandle) -> Fut,
        Fut: Send + Future<Output = Result<T>>,
    {
        let body = async {
            let mut i = 0;
            loop {
                let (tx, result) = self.with_transaction(&f).await?; //.map_err(utils::Error::from)?;
                match result {
                    Ok(result) => match tx.commit().await {
                        Ok(()) => return Ok(result),
                        Err(error) => {
                            if !self
                                .retry_on_serialization_error(&error, i)
                                .await
                            {
                                return Err(utils::Error::from(error));
                            }
                        }
                    },
                    Err(error) => {
                        tx.rollback().await.map_err(utils::Error::from)?;
                        return Err(error);
                    }
                }
                i += 1;
            }
        };

        self.run(body).await
    }

    async fn retry_on_serialization_error(
        &self,
        err: &DbErr,
        attempt: usize,
    ) -> bool {
        const SLEEPS: [f32; 10] =
            [10., 20., 40., 80., 160., 320., 640., 1280., 2560., 5120.];
        if is_serialization_err(err) && attempt < SLEEPS.len() {
            let delay = SLEEPS[attempt];
            let randomized_delay =
                delay * self.rng.lock().await.random_range(0.5..=2.0);
            warn!(
                "retrying transaction after serialization error. delay: {} ms.",
                randomized_delay
            );
            self.executor.sleep(Duration::from_millis(attempt as u64)).await;
        }

        false
    }

    pub async fn with_transaction<F, Fut, T>(
        &self,
        f: &F,
    ) -> Result<(DatabaseTransaction, Result<T>)>
    where
        F: Send + Fn(TransactionHandle) -> Fut,
        Fut: Send + Future<Output = Result<T>>,
    {
        let tx = self
            .pool
            .begin_with_config(Some(IsolationLevel::Serializable), None)
            .await?;

        let mut tx = Arc::new(Some(tx));
        let result = f(TransactionHandle(tx.clone())).await;
        let Some(tx) = Arc::get_mut(&mut tx).and_then(|tx| tx.take()) else {
            return Err(anyhow::anyhow!(
                "couldn't complete transaction because it's still in use"
            ))?;
        };

        Ok((tx, result))
    }

    async fn run<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        future.await
    }
}

pub struct TransactionHandle(pub(crate) Arc<Option<DatabaseTransaction>>);
impl Deref for TransactionHandle {
    type Target = DatabaseTransaction;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().as_ref().expect(
            "TransactionHandle is in an invalid state: inner value is None",
        )
    }
}

fn is_serialization_err(err: &DbErr) -> bool {
    match err {
        // DbErr::ConnectionAcquire(conn_acquire_err) => todo!(),
        // DbErr::TryIntoErr { from, into, source } => todo!(),
        // DbErr::Conn(runtime_err) => todo!(),
        DbErr::Exec(RuntimeErr::SqlxError(e)) => {
            e.to_string().contains("serialization")
        }
        // DbErr::ConvertFromU64(_) => todo!(),
        // DbErr::UnpackInsertId => todo!(),
        // DbErr::UpdateGetPrimaryKey => todo!(),
        // DbErr::RecordNotFound(_) => todo!(),
        // DbErr::AttrNotSet(_) => todo!(),
        // DbErr::Custom(_) => todo!(),
        // DbErr::Type(_) => todo!(),
        // DbErr::Json(_) => todo!(),
        // DbErr::Migration(_) => todo!(),
        // DbErr::RecordNotInserted => todo!(),
        // DbErr::RecordNotUpdated => todo!(),
        _ => false,
    }
}
