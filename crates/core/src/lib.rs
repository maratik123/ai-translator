//! The translation engine: segmentation, retrieval, prompting and validation.

/// The embedded set of `core`'s own schema migrations.
///
/// Applying this migrator against a database brings its schema forward by
/// running every migration under `migrations/` that the target database has
/// not yet recorded, in version order.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[cfg(test)]
mod tests {
    use super::MIGRATOR;

    #[test]
    fn embedded_migrations_satisfy_ac2() {
        let lowest = MIGRATOR
            .migrations
            .iter()
            .min_by_key(|migration| migration.version)
            .expect("the embedded set carries at least one migration");

        assert_eq!(
            lowest.version, 1,
            "the lowest-versioned migration is version 1"
        );
        assert_eq!(
            lowest.description, "vector extension",
            "the lowest-versioned migration is described vector extension \
             (sqlx replaces the file name's underscores with spaces)"
        );
        assert_eq!(
            lowest.sql.as_ref().trim(),
            "CREATE EXTENSION IF NOT EXISTS vector",
            "the lowest-versioned migration's statement is exactly the vector-extension statement"
        );
        assert!(
            MIGRATOR
                .migrations
                .iter()
                .all(|migration| migration.version >= lowest.version),
            "no migration in the set carries a version below the lowest one"
        );
    }
}
