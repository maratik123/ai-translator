//! Shared harness for `core`'s database-backed integration tests.
//!
//! One Postgres container serves the whole test binary. `main` starts it once
//! through [`Harness::start`], hands every trial a database of its own through
//! [`Harness::create_database`], and removes the container through
//! [`Harness::shutdown`] after the trial runner returns.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

/// A harness operation's error: test code, so a boxed trait object stands in
/// for a typed error per operation.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A harness operation's result.
pub type Result<T> = std::result::Result<T, Error>;

const PGVECTOR_IMAGE_NAME: &str = "pgvector/pgvector";
const PGVECTOR_IMAGE_TAG: &str = "pg18";
const ADMIN_DATABASE: &str = "postgres";
const DATABASE_NAME_PREFIX: &str = "reader_core_test_";
const POSTGRES_PORT: u16 = 5432;

/// A database created and migrated for one trial.
pub struct Database {
    /// A pool connected to the trial's own database.
    pub pool: PgPool,
    /// The database's name, as `CREATE DATABASE` assigned it.
    pub name: String,
}

/// One Postgres container shared by the whole test binary.
pub struct Harness {
    container: Mutex<Option<ContainerAsync<Postgres>>>,
    admin_pool: PgPool,
    host: String,
    port: u16,
    database_counter: AtomicU64,
    container_starts: AtomicU32,
}

impl Harness {
    /// Starts the shared container and its admin pool.
    ///
    /// # Errors
    ///
    /// Returns an error when the container fails to start or the admin pool
    /// fails to connect — including when no container runtime is reachable.
    pub async fn start() -> Result<Self> {
        let image = Postgres::default()
            .with_host_auth()
            .with_name(PGVECTOR_IMAGE_NAME)
            .with_tag(PGVECTOR_IMAGE_TAG);
        let container = image.start().await?;
        let host = container.get_host().await?.to_string();
        let port = container.get_host_port_ipv4(POSTGRES_PORT).await?;

        let admin_pool = PgPoolOptions::new()
            .connect_with(connect_options(&host, port, ADMIN_DATABASE))
            .await?;

        Ok(Self {
            container: Mutex::new(Some(container)),
            admin_pool,
            host,
            port,
            database_counter: AtomicU64::new(0),
            container_starts: AtomicU32::new(1),
        })
    }

    /// How many containers this harness has started — the trials assert it
    /// beside the server's own evidence that one server backs every database.
    pub fn container_starts(&self) -> u32 {
        self.container_starts.load(Ordering::SeqCst)
    }

    /// Creates a fresh database, applies `core`'s migrations to it, and
    /// returns a pool connected to it.
    ///
    /// The database's name is built from a fixed prefix and an atomic
    /// counter, so it is unique under any thread count and no caller-supplied
    /// text ever reaches the `CREATE DATABASE` statement.
    ///
    /// # Errors
    ///
    /// Returns an error when the database cannot be created, when a pool
    /// cannot be opened against it, or when applying the migrations fails.
    pub async fn create_database(&self) -> Result<Database> {
        let id = self.database_counter.fetch_add(1, Ordering::SeqCst);
        let name = format!("{DATABASE_NAME_PREFIX}{id}");

        // AssertSqlSafe is sound here: `name` is never caller-supplied text — it is
        // built from a fixed prefix and an atomic counter's decimal digits, so no
        // interpolated value can carry a quote or a statement separator.
        sqlx::query(sqlx::AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#)))
            .execute(&self.admin_pool)
            .await?;

        let pool = PgPoolOptions::new()
            .connect_with(connect_options(&self.host, self.port, &name))
            .await?;
        reader_core::MIGRATOR.run(&pool).await?;

        Ok(Database { pool, name })
    }

    /// Removes the container and closes the admin pool.
    ///
    /// A second call is a no-op: the container lives in a take-once slot, and
    /// this empties it before doing anything else. Driven by `main` through
    /// the runtime it owns, because the container's destructor resolves a
    /// runtime handle and `main` is not inside one.
    ///
    /// # Errors
    ///
    /// Returns an error when the container could not be removed; the result
    /// is reported rather than discarded.
    pub async fn shutdown(&self) -> Result<()> {
        let container = self
            .container
            .lock()
            .expect("the harness's own mutex is never held across a panic")
            .take();

        self.admin_pool.close().await;

        if let Some(container) = container {
            container.rm().await?;
        }

        Ok(())
    }
}

fn connect_options(host: &str, port: u16, database: &str) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(host)
        .port(port)
        .username("postgres")
        .database(database)
        .ssl_mode(PgSslMode::Disable)
}
