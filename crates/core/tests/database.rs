//! The database-backed target: one Postgres container for the whole binary,
//! a database of its own per trial. `harness = false` — this target owns its
//! own `main` rather than being discovered by `#[test]`, because the only
//! connection source the standard test attribute offers is an environment
//! variable this suite may not read.

mod schema;
mod support;

use std::sync::Arc;

use libtest_mimic::{Arguments, Failed};
use tokio::runtime::Builder;

use support::{Harness, make_trial};

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

    let mut trials = vec![
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
    trials.extend(schema::trials(&harness, &handle));

    let conclusion = libtest_mimic::run(&args, trials);
    let run_exit_code = conclusion.exit_code();

    // The runner's own verdict is captured above, before teardown, and is
    // what a clean removal exits with. A removal failure never overwrites
    // it with a panic — it is reported on stderr and forces a failing exit
    // status of its own, so a leaked container cannot exit zero even when
    // every trial passed. When the trials already failed, that verdict is
    // kept rather than replaced by the teardown failure's own status.
    match runtime.block_on(harness.shutdown()) {
        Ok(()) => run_exit_code,
        Err(err) => {
            eprintln!("failed to remove the shared container: {err}");
            if conclusion.has_failed() {
                run_exit_code
            } else {
                std::process::ExitCode::FAILURE
            }
        }
    }
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

/// The primary evidence is the harness's own count of container starts: it is
/// incremented at the site that awaits the container start, so a second
/// start anywhere in the binary is observable, and it is what can actually
/// fail. The server's own postmaster start instant is checked second, as
/// corroboration, not as a replacement: both databases here are taken inside
/// one trial, so a harness that started one container per trial would still
/// report identical instants for them. Only the count sees that failure, and
/// neither check subsumes the other.
async fn one_container_serves_the_whole_binary(harness: Arc<Harness>) -> Result<(), Failed> {
    let first = harness.create_database().await?;
    let second = harness.create_database().await?;

    let starts = harness.container_starts();
    if starts != 1 {
        return Err(format!("the harness recorded {starts} container starts, expected 1").into());
    }

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

    Ok(())
}

/// The shared path the runner's default parallelism puts the harness on,
/// driven rather than assumed: several databases requested from concurrent
/// tasks each report a distinct current database to the server itself, and
/// each carries the full applied-migration set.
///
/// The distinctness check reads `current_database()` through each database's
/// own pool rather than trusting the name the harness handed back: the
/// harness's name is a record of what it intended to create, not evidence
/// that the pool actually reached that database. Asking the server closes
/// that gap.
async fn concurrent_requests_get_distinct_databases(harness: Arc<Harness>) -> Result<(), Failed> {
    const CONCURRENT_REQUESTS: usize = 8;

    let mut tasks = Vec::with_capacity(CONCURRENT_REQUESTS);
    for _ in 0..CONCURRENT_REQUESTS {
        let harness = Arc::clone(&harness);
        tasks.push(tokio::spawn(async move { harness.create_database().await }));
    }

    let expected_versions: Vec<i64> = reader_core::MIGRATOR
        .migrations
        .iter()
        .map(|migration| migration.version)
        .collect();

    let mut reported_names = Vec::with_capacity(CONCURRENT_REQUESTS);
    for task in tasks {
        let database = task
            .await
            .map_err(|err| format!("a concurrent request's task panicked: {err}"))??;

        let reported_name: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(&database.pool)
            .await?;

        let applied_versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&database.pool)
                .await?;
        if applied_versions != expected_versions {
            return Err(format!(
                "database {:?}, reporting current_database() = {reported_name:?}, handed out \
                 under concurrency, had applied migrations {applied_versions:?}, expected \
                 {expected_versions:?}",
                database.name
            )
            .into());
        }

        reported_names.push(reported_name);
    }

    let mut distinct_names = reported_names.clone();
    distinct_names.sort();
    distinct_names.dedup();
    if distinct_names.len() != reported_names.len() {
        return Err(format!(
            "current_database() reported by the databases handed out concurrently were not all \
             distinct: {reported_names:?}"
        )
        .into());
    }

    Ok(())
}
