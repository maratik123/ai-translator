//! The catalogue trials: every assertion here queries the migrated
//! database's own catalogue, and nothing here calls a function of the
//! crate. The subject is a real server's catalogue, so it belongs in an
//! integration target rather than a unit test.

use std::collections::BTreeSet;
use std::sync::Arc;

use libtest_mimic::{Failed, Trial};
use tokio::runtime::Handle;

use crate::support::{Harness, make_trial};

/// This migration's own tables — the domain of the primary-key assertion,
/// and never the bare public schema, which also carries the migrator's own
/// bookkeeping table.
const THIS_MIGRATION_TABLES: [&str; 9] = [
    "books",
    "chapters",
    "paragraphs",
    "translations",
    "context_snapshots",
    "paragraph_embeddings",
    "paragraph_annotations",
    "positions",
    "settings",
];

pub fn trials(harness: &Arc<Harness>, handle: &Handle) -> Vec<Trial> {
    vec![
        make_trial(
            "schema_matches_the_recorded_shape",
            harness,
            handle,
            schema_matches_the_recorded_shape,
        ),
        make_trial(
            "every_table_carries_the_key_the_corpus_fixes",
            harness,
            handle,
            every_table_carries_the_key_the_corpus_fixes,
        ),
        make_trial(
            "the_named_lookups_have_an_index_of_their_own",
            harness,
            handle,
            the_named_lookups_have_an_index_of_their_own,
        ),
        make_trial(
            "the_embedding_column_is_the_declared_dimension",
            harness,
            handle,
            the_embedding_column_is_the_declared_dimension,
        ),
        make_trial(
            "no_index_over_the_embedding_column_is_hnsw",
            harness,
            handle,
            no_index_over_the_embedding_column_is_hnsw,
        ),
        make_trial(
            "the_tables_later_tasks_fill_are_created_and_empty",
            harness,
            handle,
            the_tables_later_tasks_fill_are_created_and_empty,
        ),
        make_trial(
            "every_reference_refuses_a_delete",
            harness,
            handle,
            every_reference_refuses_a_delete,
        ),
        make_trial(
            "the_schema_carries_no_check_constraint",
            harness,
            handle,
            the_schema_carries_no_check_constraint,
        ),
    ]
}

type ColumnTuple = (String, String, String, bool, Option<String>);

/// The golden this migration's own shape must match: for every table of the
/// public schema except the migrator's bookkeeping table, the tuple (table,
/// column, the catalogue's own formatted type, required, default
/// expression), compared as a sorted set against the expected table in both
/// directions.
async fn schema_matches_the_recorded_shape(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let actual: BTreeSet<ColumnTuple> = sqlx::query_as::<_, ColumnTuple>(
        "SELECT c.relname, a.attname, format_type(a.atttypid, a.atttypmod), a.attnotnull, \
                pg_get_expr(d.adbin, d.adrelid) \
         FROM pg_attribute a \
         JOIN pg_class c ON c.oid = a.attrelid \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
         WHERE n.nspname = 'public' AND c.relkind = 'r' AND c.relname <> '_sqlx_migrations' \
           AND a.attnum > 0 AND NOT a.attisdropped",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .collect();

    let expected: BTreeSet<ColumnTuple> = expected_columns()
        .into_iter()
        .map(|(table, column, formatted_type, required, default)| {
            (
                table.to_string(),
                column.to_string(),
                formatted_type.to_string(),
                required,
                default.map(str::to_string),
            )
        })
        .collect();

    compare_sets("the recorded column shape", &expected, &actual)
}

fn expected_columns() -> Vec<(
    &'static str,
    &'static str,
    &'static str,
    bool,
    Option<&'static str>,
)> {
    vec![
        (
            "books",
            "id",
            "bigint",
            true,
            Some("nextval('books_id_seq'::regclass)"),
        ),
        ("books", "title", "text", true, None),
        ("books", "author", "text", false, None),
        ("books", "lang_src", "text", true, None),
        ("books", "lang_dst", "text", true, None),
        ("books", "format", "text", true, None),
        ("books", "cover_path", "text", false, None),
        ("books", "added_at", "timestamp with time zone", true, None),
        (
            "chapters",
            "id",
            "bigint",
            true,
            Some("nextval('chapters_id_seq'::regclass)"),
        ),
        ("chapters", "book_id", "bigint", true, None),
        ("chapters", "idx", "integer", true, None),
        ("chapters", "title", "text", false, None),
        ("chapters", "css", "text", false, None),
        (
            "paragraphs",
            "id",
            "bigint",
            true,
            Some("nextval('paragraphs_id_seq'::regclass)"),
        ),
        ("paragraphs", "chapter_id", "bigint", true, None),
        ("paragraphs", "idx", "integer", true, None),
        ("paragraphs", "kind", "text", true, None),
        ("paragraphs", "html", "text", true, None),
        ("paragraphs", "text", "text", true, None),
        ("paragraphs", "stable_hash", "bytea", true, None),
        ("translations", "paragraph_id", "bigint", true, None),
        ("translations", "cache_key", "bytea", true, None),
        ("translations", "model", "text", true, None),
        ("translations", "prompt_version", "integer", true, None),
        ("translations", "context_version", "integer", true, None),
        ("translations", "text", "text", true, None),
        (
            "translations",
            "created_at",
            "timestamp with time zone",
            true,
            None,
        ),
        ("context_snapshots", "book_id", "bigint", true, None),
        ("context_snapshots", "version", "integer", true, None),
        (
            "context_snapshots",
            "upto_paragraph_id",
            "bigint",
            true,
            None,
        ),
        ("context_snapshots", "summary", "text", true, None),
        ("context_snapshots", "glossary", "jsonb", true, None),
        ("context_snapshots", "model", "text", true, None),
        (
            "context_snapshots",
            "created_at",
            "timestamp with time zone",
            true,
            None,
        ),
        ("paragraph_embeddings", "paragraph_id", "bigint", true, None),
        ("paragraph_embeddings", "model", "text", true, None),
        (
            "paragraph_embeddings",
            "embedding",
            "vector(1024)",
            true,
            None,
        ),
        (
            "paragraph_annotations",
            "paragraph_id",
            "bigint",
            true,
            None,
        ),
        (
            "paragraph_annotations",
            "context_version",
            "integer",
            true,
            None,
        ),
        (
            "paragraph_annotations",
            "annotated_text",
            "text",
            true,
            None,
        ),
        ("paragraph_annotations", "uncertain", "boolean", true, None),
        ("positions", "book_id", "bigint", true, None),
        ("positions", "paragraph_id", "bigint", true, None),
        (
            "positions",
            "updated_at",
            "timestamp with time zone",
            true,
            None,
        ),
        ("settings", "key", "text", true, None),
        ("settings", "value", "jsonb", true, None),
    ]
}

fn compare_sets<T: Ord + std::fmt::Debug>(
    label: &str,
    expected: &BTreeSet<T>,
    actual: &BTreeSet<T>,
) -> Result<(), Failed> {
    let missing: Vec<&T> = expected.difference(actual).collect();
    let extra: Vec<&T> = actual.difference(expected).collect();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }
    Err(
        format!("{label} disagrees with the catalogue — missing: {missing:?}, extra: {extra:?}")
            .into(),
    )
}

/// A second golden beside the column shape above: for each table this
/// migration creates, its ordered primary-key column list, compared
/// against the catalogue as a set in both directions.
async fn every_table_carries_the_key_the_corpus_fixes(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let rows: Vec<(String, Vec<String>)> = sqlx::query_as(
        "SELECT c.relname, \
                (SELECT array_agg(att.attname ORDER BY k.ord) \
                 FROM unnest(i.indkey) WITH ORDINALITY AS k(attnum, ord) \
                 JOIN pg_attribute att ON att.attrelid = i.indrelid AND att.attnum = k.attnum) \
         FROM pg_index i \
         JOIN pg_class c ON c.oid = i.indrelid \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE i.indisprimary AND n.nspname = 'public' AND c.relname = ANY($1)",
    )
    .bind(&THIS_MIGRATION_TABLES[..])
    .fetch_all(pool)
    .await?;

    let actual: BTreeSet<(String, Vec<String>)> = rows.into_iter().collect();
    let expected: BTreeSet<(String, Vec<String>)> = expected_keys()
        .into_iter()
        .map(|(table, columns)| {
            (
                table.to_string(),
                columns.iter().map(|c| c.to_string()).collect(),
            )
        })
        .collect();

    compare_sets("the primary-key column lists", &expected, &actual)
}

fn expected_keys() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("books", vec!["id"]),
        ("chapters", vec!["id"]),
        ("paragraphs", vec!["id"]),
        (
            "translations",
            vec!["paragraph_id", "cache_key", "context_version"],
        ),
        ("context_snapshots", vec!["book_id", "version"]),
        ("paragraph_embeddings", vec!["paragraph_id"]),
        (
            "paragraph_annotations",
            vec!["paragraph_id", "context_version"],
        ),
        ("positions", vec!["book_id"]),
        ("settings", vec!["key"]),
    ]
}

/// For each of the reading-order and snapshot lookups this migration names,
/// assert that an index exists whose access method is btree and whose
/// ordered key-column list is the expected one.
async fn the_named_lookups_have_an_index_of_their_own(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let lookups: [(&str, &[&str]); 3] = [
        ("paragraphs_chapter_id_idx_uniq", &["chapter_id", "idx"]),
        ("translations_paragraph_id_idx", &["paragraph_id"]),
        (
            "context_snapshots_book_id_upto_paragraph_id_idx",
            &["book_id", "upto_paragraph_id"],
        ),
    ];

    for (index_name, expected_columns) in lookups {
        let row: Option<(String, Vec<String>)> = sqlx::query_as(
            "SELECT am.amname, \
                    (SELECT array_agg(att.attname ORDER BY k.ord) \
                     FROM unnest(i.indkey) WITH ORDINALITY AS k(attnum, ord) \
                     JOIN pg_attribute att ON att.attrelid = i.indrelid AND att.attnum = k.attnum) \
             FROM pg_index i \
             JOIN pg_class c ON c.oid = i.indexrelid \
             JOIN pg_am am ON am.oid = c.relam \
             WHERE c.relname = $1",
        )
        .bind(index_name)
        .fetch_optional(pool)
        .await?;

        let Some((access_method, key_columns)) = row else {
            return Err(format!("no index named {index_name} exists").into());
        };
        if access_method != "btree" {
            return Err(format!(
                "index {index_name} has access method {access_method}, expected btree"
            )
            .into());
        }
        if key_columns != expected_columns {
            return Err(format!(
                "index {index_name} has key columns {key_columns:?}, expected {expected_columns:?}"
            )
            .into());
        }
    }

    Ok(())
}

/// The formatted type of the embedding column is the 1024-dimension vector
/// type, read with the catalogue's type formatter.
async fn the_embedding_column_is_the_declared_dimension(
    harness: Arc<Harness>,
) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let formatted_type: String = sqlx::query_scalar(
        "SELECT format_type(a.atttypid, a.atttypmod) \
         FROM pg_attribute a \
         JOIN pg_class c ON c.oid = a.attrelid \
         WHERE c.relname = 'paragraph_embeddings' AND a.attname = 'embedding'",
    )
    .fetch_one(pool)
    .await?;

    if formatted_type != "vector(1024)" {
        return Err(format!(
            "the embedding column's formatted type is {formatted_type:?}, expected \"vector(1024)\""
        )
        .into());
    }

    Ok(())
}

/// The claim is that nothing of a kind exists, so the check has to be able
/// to see one: the access methods of every index on the embedding table are
/// asserted exactly, and the expected set is the primary key's btree alone.
async fn no_index_over_the_embedding_column_is_hnsw(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let access_methods: BTreeSet<String> = sqlx::query_scalar(
        "SELECT am.amname \
         FROM pg_index i \
         JOIN pg_class c ON c.oid = i.indexrelid \
         JOIN pg_am am ON am.oid = c.relam \
         WHERE i.indrelid = 'paragraph_embeddings'::regclass",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .collect();

    let expected: BTreeSet<String> = ["btree".to_string()].into_iter().collect();
    compare_sets(
        "the embedding table's index access methods",
        &expected,
        &access_methods,
    )
}

/// The embedding and annotation tables are present in the catalogue and
/// each holds no row after the migration.
async fn the_tables_later_tasks_fill_are_created_and_empty(
    harness: Arc<Harness>,
) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let checks: [(&str, &str, &str); 2] = [
        (
            "paragraph_embeddings",
            "public.paragraph_embeddings",
            "SELECT count(*) FROM paragraph_embeddings",
        ),
        (
            "paragraph_annotations",
            "public.paragraph_annotations",
            "SELECT count(*) FROM paragraph_annotations",
        ),
    ];

    for (table, qualified_name, count_query) in checks {
        let present: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(qualified_name)
            .fetch_one(pool)
            .await?;
        if !present {
            return Err(format!("table {table} is not present in the catalogue").into());
        }

        let row_count: i64 = sqlx::query_scalar(count_query).fetch_one(pool).await?;
        if row_count != 0 {
            return Err(format!("table {table} holds {row_count} row(s), expected none").into());
        }
    }

    Ok(())
}

/// Over every foreign key in the public schema, the recorded delete rule is
/// restrict. Schema-wide rather than table by table, so a reference added
/// later by an unrelated migration cannot introduce a cascade unnoticed.
async fn every_reference_refuses_a_delete(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT conname, confdeltype::text \
         FROM pg_constraint \
         WHERE contype = 'f' AND connamespace = 'public'::regnamespace",
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Err("no foreign key was found in the public schema".into());
    }

    let non_restrict: Vec<(String, String)> = rows
        .into_iter()
        .filter(|(_, delete_type)| delete_type != "r")
        .collect();
    if !non_restrict.is_empty() {
        return Err(format!(
            "reference(s) not carrying the restrict delete rule: {non_restrict:?}"
        )
        .into());
    }

    Ok(())
}

/// The catalogue holds no constraint of the check kind: nothing in the
/// schema judges what a value contains. Runs over the bare public schema
/// with no exclusion, because the migrator's bookkeeping table declares no
/// check constraint of its own.
async fn the_schema_carries_no_check_constraint(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let check_constraints: Vec<String> = sqlx::query_scalar(
        "SELECT conname FROM pg_constraint \
         WHERE contype = 'c' AND connamespace = 'public'::regnamespace",
    )
    .fetch_all(pool)
    .await?;

    if !check_constraints.is_empty() {
        return Err(format!(
            "the public schema carries check constraint(s): {check_constraints:?}"
        )
        .into());
    }

    Ok(())
}
