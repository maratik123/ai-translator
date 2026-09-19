//! The database-backed target: one Postgres container for the whole binary,
//! a database of its own per trial. `harness = false` — this target owns its
//! own `main` rather than being discovered by `#[test]`, because the only
//! connection source the standard test attribute offers is an environment
//! variable this suite may not read.

mod support;

use std::future::Future;
use std::sync::Arc;

use libtest_mimic::{Arguments, Failed, Trial};
use tokio::runtime::{Builder, Handle};

use support::Harness;

fn main() -> std::process::ExitCode {
    let args = Arguments::from_args();

    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build the tokio runtime the harness runs on");
    let handle = runtime.handle().clone();

    let harness = runtime
        .block_on(Harness::start())
        .expect("failed to start the shared Postgres container");
    let harness = Arc::new(harness);

    let trials = vec![
        make_trial(
            "vector_type_is_usable",
            &harness,
            &handle,
            vector_type_is_usable,
        ),
        make_trial(
            "migrations_are_applied_to_every_database",
            &harness,
            &handle,
            migrations_are_applied_to_every_database,
        ),
        make_trial(
            "each_trial_gets_its_own_database",
            &harness,
            &handle,
            each_trial_gets_its_own_database,
        ),
        make_trial(
            "one_container_serves_the_whole_binary",
            &harness,
            &handle,
            one_container_serves_the_whole_binary,
        ),
        make_trial(
            "concurrent_requests_get_distinct_databases",
            &harness,
            &handle,
            concurrent_requests_get_distinct_databases,
        ),
    ];

    let conclusion = libtest_mimic::run(&args, trials);

    runtime
        .block_on(harness.shutdown())
        .expect("failed to remove the shared container");

    conclusion.exit_code()
}

/// Wraps a trial body — an async function taking the shared harness — into a
/// [`Trial`] that runs it on `main`'s own runtime through a cloned [`Handle`].
/// The closure owns an `Arc` clone rather than a borrow, because
/// `Trial::test` requires its runner to be `'static`.
fn make_trial<F, Fut>(name: &'static str, harness: &Arc<Harness>, handle: &Handle, body: F) -> Trial
where
    F: FnOnce(Arc<Harness>) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), Failed>>,
{
    let harness = Arc::clone(harness);
    let handle = handle.clone();
    Trial::test(name, move || handle.block_on(body(harness)))
}

/// A freshly migrated database really carries a usable `vector` type: a
/// bracketed vector literal round-trips through the type unchanged.
async fn vector_type_is_usable(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let literal = "[1,2,3]";

    let round_tripped: String = sqlx::query_scalar("SELECT $1::vector::text")
        .bind(literal)
        .fetch_one(&database.pool)
        .await?;

    if round_tripped != literal {
        return Err(format!(
            "the vector literal round-tripped to {round_tripped:?}, expected {literal:?}"
        )
        .into());
    }

    Ok(())
}

/// A freshly created database carries both halves of "migrated": the
/// extension is present in the catalogue, and the applied-migration table
/// records exactly the versions the embedded set carries.
async fn migrations_are_applied_to_every_database(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;

    let extension_present: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(&database.pool)
            .await?;
    if !extension_present {
        return Err("the vector extension is not present in the catalogue".into());
    }

    let applied_versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&database.pool)
            .await?;
    let expected_versions: Vec<i64> = reader_core::MIGRATOR
        .migrations
        .iter()
        .map(|migration| migration.version)
        .collect();

    if applied_versions != expected_versions {
        return Err(format!(
            "applied migrations were {applied_versions:?}, expected {expected_versions:?}"
        )
        .into());
    }

    Ok(())
}

/// Two databases handed to one trial are isolated: distinct names, and a
/// table created in one is absent from the other.
async fn each_trial_gets_its_own_database(harness: Arc<Harness>) -> Result<(), Failed> {
    let first = harness.create_database().await?;
    let second = harness.create_database().await?;

    let first_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&first.pool)
        .await?;
    let second_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&second.pool)
        .await?;
    if first_name == second_name {
        return Err(format!("both databases reported the same name {first_name:?}").into());
    }

    sqlx::query("CREATE TABLE only_in_first (id integer)")
        .execute(&first.pool)
        .await?;

    let table_leaked: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'only_in_first')",
    )
    .fetch_one(&second.pool)
    .await?;
    if table_leaked {
        return Err("a table created in one database is visible from the other".into());
    }

    Ok(())
}

/// The two databases of a fresh pair report the identical postmaster start
/// instant, which is the server's own evidence that one server backs them
/// both; the harness's own start count is asserted beside it as the cheap
/// cross-check.
async fn one_container_serves_the_whole_binary(harness: Arc<Harness>) -> Result<(), Failed> {
    let first = harness.create_database().await?;
    let second = harness.create_database().await?;

    let first_start: String = sqlx::query_scalar("SELECT pg_postmaster_start_time()::text")
        .fetch_one(&first.pool)
        .await?;
    let second_start: String = sqlx::query_scalar("SELECT pg_postmaster_start_time()::text")
        .fetch_one(&second.pool)
        .await?;
    if first_start != second_start {
        return Err(format!(
            "the two databases reported different postmaster start instants: {first_start} vs {second_start}"
        )
        .into());
    }

    let starts = harness.container_starts();
    if starts != 1 {
        return Err(format!("the harness recorded {starts} container starts, expected 1").into());
    }

    Ok(())
}

/// The shared path the runner's default parallelism puts the harness on,
/// driven rather than assumed: several databases requested from concurrent
/// tasks are all distinct and all migrated.
async fn concurrent_requests_get_distinct_databases(harness: Arc<Harness>) -> Result<(), Failed> {
    const CONCURRENT_REQUESTS: usize = 8;

    let mut tasks = Vec::with_capacity(CONCURRENT_REQUESTS);
    for _ in 0..CONCURRENT_REQUESTS {
        let harness = Arc::clone(&harness);
        tasks.push(tokio::spawn(async move { harness.create_database().await }));
    }

    let mut names = Vec::with_capacity(CONCURRENT_REQUESTS);
    for task in tasks {
        let database = task
            .await
            .map_err(|err| format!("a concurrent request's task panicked: {err}"))??;

        let extension_present: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')",
        )
        .fetch_one(&database.pool)
        .await?;
        if !extension_present {
            return Err(format!(
                "database {:?} handed out under concurrency was not migrated",
                database.name
            )
            .into());
        }

        names.push(database.name);
    }

    let mut distinct_names = names.clone();
    distinct_names.sort();
    distinct_names.dedup();
    if distinct_names.len() != names.len() {
        return Err(format!(
            "database names handed out concurrently were not all distinct: {names:?}"
        )
        .into());
    }

    Ok(())
}
