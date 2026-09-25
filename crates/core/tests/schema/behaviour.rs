//! The behaviour trials: every assertion here is an insert or a delete
//! against the migrated database, and the verdict is the server's answer to
//! that statement.

use std::sync::Arc;

use libtest_mimic::{Failed, Trial};
use tokio::runtime::Handle;

use crate::schema;
use crate::support::{Harness, make_trial};

pub fn trials(harness: &Arc<Harness>, handle: &Handle) -> Vec<Trial> {
    vec![
        make_trial(
            "context_versions_of_one_cache_key_coexist",
            harness,
            handle,
            context_versions_of_one_cache_key_coexist,
        ),
        make_trial(
            "a_repeated_translation_key_is_refused",
            harness,
            handle,
            a_repeated_translation_key_is_refused,
        ),
        make_trial(
            "a_cache_lookup_naming_no_context_version_finds_the_row",
            harness,
            handle,
            a_cache_lookup_naming_no_context_version_finds_the_row,
        ),
        make_trial(
            "a_vector_of_another_dimension_is_refused",
            harness,
            handle,
            a_vector_of_another_dimension_is_refused,
        ),
        make_trial(
            "a_position_is_taken_once_within_its_parent",
            harness,
            handle,
            a_position_is_taken_once_within_its_parent,
        ),
        make_trial(
            "a_row_missing_a_required_value_is_refused",
            harness,
            handle,
            a_row_missing_a_required_value_is_refused,
        ),
        make_trial(
            "the_database_judges_presence_not_content",
            harness,
            handle,
            the_database_judges_presence_not_content,
        ),
        make_trial(
            "a_value_outside_the_supported_set_is_stored_as_given",
            harness,
            handle,
            a_value_outside_the_supported_set_is_stored_as_given,
        ),
        make_trial(
            "deleting_a_book_is_refused_while_anything_references_it",
            harness,
            handle,
            deleting_a_book_is_refused_while_anything_references_it,
        ),
        make_trial(
            "a_work_with_no_divisions_is_stored_as_one_untitled_chapter",
            harness,
            handle,
            a_work_with_no_divisions_is_stored_as_one_untitled_chapter,
        ),
    ]
}

/// Two translations of one paragraph sharing a cache key and differing in
/// context version are both stored and both readable.
async fn context_versions_of_one_cache_key_coexist(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;
    let fixture = schema::insert_book_chapter_paragraph(pool).await?;
    let cache_key = vec![7u8];

    for context_version in [1_i32, 2] {
        sqlx::query(
            "INSERT INTO translations \
             (paragraph_id, cache_key, model, prompt_version, context_version, text, created_at) \
             VALUES ($1, $2, 'm', 1, $3, 't', now())",
        )
        .bind(fixture.paragraph_id)
        .bind(&cache_key)
        .bind(context_version)
        .execute(pool)
        .await?;
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM translations WHERE paragraph_id = $1 AND cache_key = $2",
    )
    .bind(fixture.paragraph_id)
    .bind(&cache_key)
    .fetch_one(pool)
    .await?;

    if count != 2 {
        return Err(format!("expected 2 stored translations, found {count}").into());
    }
    Ok(())
}

/// A third row repeating paragraph, cache key and context version is
/// refused with the unique-violation SQLSTATE, and the stored rows are
/// unchanged afterwards.
async fn a_repeated_translation_key_is_refused(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;
    let fixture = schema::insert_book_chapter_paragraph(pool).await?;
    let cache_key = vec![9u8];

    sqlx::query(
        "INSERT INTO translations \
         (paragraph_id, cache_key, model, prompt_version, context_version, text, created_at) \
         VALUES ($1, $2, 'm', 1, 1, 't', now())",
    )
    .bind(fixture.paragraph_id)
    .bind(&cache_key)
    .execute(pool)
    .await?;

    let result = sqlx::query(
        "INSERT INTO translations \
         (paragraph_id, cache_key, model, prompt_version, context_version, text, created_at) \
         VALUES ($1, $2, 'm2', 2, 1, 't2', now())",
    )
    .bind(fixture.paragraph_id)
    .bind(&cache_key)
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23505")?;

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM translations WHERE paragraph_id = $1 AND cache_key = $2",
    )
    .bind(fixture.paragraph_id)
    .bind(&cache_key)
    .fetch_one(pool)
    .await?;
    if count != 1 {
        return Err(format!("expected 1 stored row after the refusal, found {count}").into());
    }
    Ok(())
}

/// The corpus's own pre-check shape, a lookup by paragraph and cache key
/// with no context version named, returns a row.
async fn a_cache_lookup_naming_no_context_version_finds_the_row(
    harness: Arc<Harness>,
) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;
    let fixture = schema::insert_book_chapter_paragraph(pool).await?;
    let cache_key = vec![3u8];

    sqlx::query(
        "INSERT INTO translations \
         (paragraph_id, cache_key, model, prompt_version, context_version, text, created_at) \
         VALUES ($1, $2, 'm', 1, 5, 't', now())",
    )
    .bind(fixture.paragraph_id)
    .bind(&cache_key)
    .execute(pool)
    .await?;

    let found: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM translations WHERE paragraph_id = $1 AND cache_key = $2)",
    )
    .bind(fixture.paragraph_id)
    .bind(&cache_key)
    .fetch_one(pool)
    .await?;

    if !found {
        return Err("the cache lookup by paragraph and cache key alone found no row".into());
    }
    Ok(())
}

/// A 1024-element vector is stored and reads back at that dimension; a
/// vector of another dimension is refused with the data-exception SQLSTATE,
/// and no row lands.
async fn a_vector_of_another_dimension_is_refused(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;
    let fixture = schema::insert_book_chapter_paragraph(pool).await?;

    let ok_vector = schema::zero_vector_literal(schema::EMBEDDING_DIMENSION);
    sqlx::query(
        "INSERT INTO paragraph_embeddings (paragraph_id, model, embedding) \
         VALUES ($1, 'm', $2::vector)",
    )
    .bind(fixture.paragraph_id)
    .bind(&ok_vector)
    .execute(pool)
    .await?;

    let round_tripped: String = sqlx::query_scalar(
        "SELECT embedding::text FROM paragraph_embeddings WHERE paragraph_id = $1",
    )
    .bind(fixture.paragraph_id)
    .fetch_one(pool)
    .await?;
    if round_tripped != ok_vector {
        return Err(format!(
            "the stored embedding round-tripped to {round_tripped:?}, expected {ok_vector:?}"
        )
        .into());
    }

    let second_paragraph_id = schema::insert_paragraph(pool, fixture.chapter_id, 1).await?;
    let wrong_vector = schema::zero_vector_literal(schema::EMBEDDING_DIMENSION - 1);
    let result = sqlx::query(
        "INSERT INTO paragraph_embeddings (paragraph_id, model, embedding) \
         VALUES ($1, 'm', $2::vector)",
    )
    .bind(second_paragraph_id)
    .bind(&wrong_vector)
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "22000")?;

    let stored: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM paragraph_embeddings WHERE paragraph_id = $1)",
    )
    .bind(second_paragraph_id)
    .fetch_one(pool)
    .await?;
    if stored {
        return Err("a vector of the wrong dimension was stored".into());
    }
    Ok(())
}

/// A second chapter at a position already taken in its book, and a second
/// paragraph at a position already taken in its chapter, are each refused;
/// the same positions under a different parent are accepted.
async fn a_position_is_taken_once_within_its_parent(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let book_id = schema::insert_book(pool).await?;
    schema::insert_chapter(pool, book_id, 0).await?;
    let result = sqlx::query("INSERT INTO chapters (book_id, idx) VALUES ($1, 0)")
        .bind(book_id)
        .execute(pool)
        .await;
    schema::assert_sqlstate(result, "23505")
        .map_err(|err| format!("second chapter, same book, same position: {err}"))?;

    let other_book_id = schema::insert_book(pool).await?;
    sqlx::query("INSERT INTO chapters (book_id, idx) VALUES ($1, 0)")
        .bind(other_book_id)
        .execute(pool)
        .await
        .map_err(|err| format!("same chapter position under a different book: {err}"))?;

    let chapter_id = schema::insert_chapter(pool, book_id, 1).await?;
    schema::insert_paragraph(pool, chapter_id, 0).await?;
    let result = sqlx::query(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES ($1, 0, 'p', '<p></p>', 't', $2)",
    )
    .bind(chapter_id)
    .bind(vec![0u8])
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23505")
        .map_err(|err| format!("second paragraph, same chapter, same position: {err}"))?;

    let other_chapter_id = schema::insert_chapter(pool, book_id, 2).await?;
    sqlx::query(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES ($1, 0, 'p', '<p></p>', 't', $2)",
    )
    .bind(other_chapter_id)
    .bind(vec![0u8])
    .execute(pool)
    .await
    .map_err(|err| format!("same paragraph position under a different chapter: {err}"))?;

    Ok(())
}

/// A paragraph with no text, a chapter with no book, a paragraph with no
/// chapter, a translation with no cache key, a snapshot with no window
/// bound, a position with no paragraph — each refused with the not-null
/// SQLSTATE; a child row naming a parent that does not exist is refused
/// with the foreign-key-violation SQLSTATE instead.
async fn a_row_missing_a_required_value_is_refused(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;
    let fixture = schema::insert_book_chapter_paragraph(pool).await?;

    let result = sqlx::query(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES ($1, 1, 'p', '<p></p>', $2, $3)",
    )
    .bind(fixture.chapter_id)
    .bind(None::<String>)
    .bind(vec![0u8])
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23502")
        .map_err(|err| format!("paragraph with no text: {err}"))?;

    let result = sqlx::query("INSERT INTO chapters (book_id, idx) VALUES ($1, 5)")
        .bind(None::<i64>)
        .execute(pool)
        .await;
    schema::assert_sqlstate(result, "23502")
        .map_err(|err| format!("chapter with no book: {err}"))?;

    let result = sqlx::query(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES ($1, 2, 'p', '<p></p>', 't', $2)",
    )
    .bind(None::<i64>)
    .bind(vec![0u8])
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23502")
        .map_err(|err| format!("paragraph with no chapter: {err}"))?;

    let result = sqlx::query(
        "INSERT INTO translations \
         (paragraph_id, cache_key, model, prompt_version, context_version, text, created_at) \
         VALUES ($1, $2, 'm', 1, 1, 't', now())",
    )
    .bind(fixture.paragraph_id)
    .bind(None::<Vec<u8>>)
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23502")
        .map_err(|err| format!("translation with no cache key: {err}"))?;

    let result = sqlx::query(
        "INSERT INTO context_snapshots \
         (book_id, version, upto_paragraph_id, summary, glossary, model, created_at) \
         VALUES ($1, 1, $2, 's', '{}'::jsonb, 'm', now())",
    )
    .bind(fixture.book_id)
    .bind(None::<i64>)
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23502")
        .map_err(|err| format!("snapshot with no window bound: {err}"))?;

    let result = sqlx::query(
        "INSERT INTO positions (book_id, paragraph_id, updated_at) VALUES ($1, $2, now())",
    )
    .bind(fixture.book_id)
    .bind(None::<i64>)
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23502")
        .map_err(|err| format!("position with no paragraph: {err}"))?;

    let missing_book_id = fixture.book_id + 1_000_000;
    let result = sqlx::query("INSERT INTO chapters (book_id, idx) VALUES ($1, 9)")
        .bind(missing_book_id)
        .execute(pool)
        .await;
    schema::assert_sqlstate(result, "23503")
        .map_err(|err| format!("chapter naming a book that does not exist: {err}"))?;

    Ok(())
}

/// A paragraph whose text is the empty string, a negative position, an
/// empty cache key and an empty glossary object are each stored unchanged.
async fn the_database_judges_presence_not_content(harness: Arc<Harness>) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;
    let fixture = schema::insert_book_chapter_paragraph(pool).await?;

    let paragraph_id: i64 = sqlx::query_scalar(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES ($1, 1, 'p', '<p></p>', $2, $3) RETURNING id",
    )
    .bind(fixture.chapter_id)
    .bind("")
    .bind(vec![0u8])
    .fetch_one(pool)
    .await?;
    let stored_text: String = sqlx::query_scalar("SELECT text FROM paragraphs WHERE id = $1")
        .bind(paragraph_id)
        .fetch_one(pool)
        .await?;
    if !stored_text.is_empty() {
        return Err(format!("the empty paragraph text round-tripped to {stored_text:?}").into());
    }

    let chapter_id: i64 =
        sqlx::query_scalar("INSERT INTO chapters (book_id, idx) VALUES ($1, -1) RETURNING id")
            .bind(fixture.book_id)
            .fetch_one(pool)
            .await?;
    let stored_idx: i32 = sqlx::query_scalar("SELECT idx FROM chapters WHERE id = $1")
        .bind(chapter_id)
        .fetch_one(pool)
        .await?;
    if stored_idx != -1 {
        return Err(format!("the negative position round-tripped to {stored_idx}").into());
    }

    sqlx::query(
        "INSERT INTO translations \
         (paragraph_id, cache_key, model, prompt_version, context_version, text, created_at) \
         VALUES ($1, $2, 'm', 1, 1, 't', now())",
    )
    .bind(fixture.paragraph_id)
    .bind(Vec::<u8>::new())
    .execute(pool)
    .await?;
    let stored_cache_key: Vec<u8> = sqlx::query_scalar(
        "SELECT cache_key FROM translations WHERE paragraph_id = $1 AND context_version = 1",
    )
    .bind(fixture.paragraph_id)
    .fetch_one(pool)
    .await?;
    if !stored_cache_key.is_empty() {
        return Err(format!("the empty cache key round-tripped to {stored_cache_key:?}").into());
    }

    sqlx::query(
        "INSERT INTO context_snapshots \
         (book_id, version, upto_paragraph_id, summary, glossary, model, created_at) \
         VALUES ($1, 1, $2, 's', $3::jsonb, 'm', now())",
    )
    .bind(fixture.book_id)
    .bind(fixture.paragraph_id)
    .bind("{}")
    .execute(pool)
    .await?;
    let stored_glossary: String = sqlx::query_scalar(
        "SELECT glossary::text FROM context_snapshots WHERE book_id = $1 AND version = 1",
    )
    .bind(fixture.book_id)
    .fetch_one(pool)
    .await?;
    if stored_glossary != "{}" {
        return Err(format!("the empty glossary round-tripped to {stored_glossary:?}").into());
    }

    Ok(())
}

/// A paragraph kind, a book format, a source language and a target
/// language each written with a value the project does not support are
/// read back unchanged.
async fn a_value_outside_the_supported_set_is_stored_as_given(
    harness: Arc<Harness>,
) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let book_id: i64 = sqlx::query_scalar(
        "INSERT INTO books (title, lang_src, lang_dst, format, added_at) \
         VALUES ('t', $1, $2, $3, now()) RETURNING id",
    )
    .bind("xx")
    .bind("yy")
    .bind("nonexistent-format")
    .fetch_one(pool)
    .await?;

    let (lang_src, lang_dst, format): (String, String, String) =
        sqlx::query_as("SELECT lang_src, lang_dst, format FROM books WHERE id = $1")
            .bind(book_id)
            .fetch_one(pool)
            .await?;
    if (lang_src.as_str(), lang_dst.as_str(), format.as_str()) != ("xx", "yy", "nonexistent-format")
    {
        return Err(format!(
            "the unsupported book values round-tripped to {lang_src:?}, {lang_dst:?}, {format:?}"
        )
        .into());
    }

    let chapter_id = schema::insert_chapter(pool, book_id, 0).await?;
    let paragraph_id: i64 = sqlx::query_scalar(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES ($1, 0, $2, '<p></p>', 't', $3) RETURNING id",
    )
    .bind(chapter_id)
    .bind("nonexistent-kind")
    .bind(vec![0u8])
    .fetch_one(pool)
    .await?;

    let stored_kind: String = sqlx::query_scalar("SELECT kind FROM paragraphs WHERE id = $1")
        .bind(paragraph_id)
        .fetch_one(pool)
        .await?;
    if stored_kind != "nonexistent-kind" {
        return Err(
            format!("the unsupported paragraph kind round-tripped to {stored_kind:?}").into(),
        );
    }

    Ok(())
}

/// A whole book is built with a row in every referencing table. The
/// delete is refused with the restrict SQLSTATE; every row is then found
/// still present; and the deletion succeeds once the caller has removed
/// the content itself, in dependency order. Three narrower cases precede
/// it, each with a single remaining reference, so the failure message
/// names which one held the book: a chapter, a context snapshot, a
/// reading position.
async fn deleting_a_book_is_refused_while_anything_references_it(
    harness: Arc<Harness>,
) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    {
        let book_id = schema::insert_book(pool).await?;
        schema::insert_chapter(pool, book_id, 0).await?;
        let result = sqlx::query("DELETE FROM books WHERE id = $1")
            .bind(book_id)
            .execute(pool)
            .await;
        schema::assert_sqlstate(result, "23001")
            .map_err(|err| format!("chapter as the sole reference: {err}"))?;
        assert_book_still_present(pool, book_id, "chapter case").await?;
    }

    {
        let book_id = schema::insert_book(pool).await?;
        sqlx::query(
            "INSERT INTO context_snapshots \
             (book_id, version, upto_paragraph_id, summary, glossary, model, created_at) \
             VALUES ($1, 1, 1, 's', '{}'::jsonb, 'm', now())",
        )
        .bind(book_id)
        .execute(pool)
        .await?;
        let result = sqlx::query("DELETE FROM books WHERE id = $1")
            .bind(book_id)
            .execute(pool)
            .await;
        schema::assert_sqlstate(result, "23001")
            .map_err(|err| format!("context snapshot as the sole reference: {err}"))?;
        assert_book_still_present(pool, book_id, "context snapshot case").await?;
    }

    {
        let book_id = schema::insert_book(pool).await?;
        sqlx::query(
            "INSERT INTO positions (book_id, paragraph_id, updated_at) VALUES ($1, 1, now())",
        )
        .bind(book_id)
        .execute(pool)
        .await?;
        let result = sqlx::query("DELETE FROM books WHERE id = $1")
            .bind(book_id)
            .execute(pool)
            .await;
        schema::assert_sqlstate(result, "23001")
            .map_err(|err| format!("reading position as the sole reference: {err}"))?;
        assert_book_still_present(pool, book_id, "reading position case").await?;
    }

    let whole = schema::insert_whole_book(pool).await?;
    let result = sqlx::query("DELETE FROM books WHERE id = $1")
        .bind(whole.book_id)
        .execute(pool)
        .await;
    schema::assert_sqlstate(result, "23001").map_err(|err| format!("whole book: {err}"))?;

    let counts: [(&str, i64); 8] = [
        (
            "books",
            sqlx::query_scalar("SELECT count(*) FROM books WHERE id = $1")
                .bind(whole.book_id)
                .fetch_one(pool)
                .await?,
        ),
        (
            "chapters",
            sqlx::query_scalar("SELECT count(*) FROM chapters WHERE book_id = $1")
                .bind(whole.book_id)
                .fetch_one(pool)
                .await?,
        ),
        (
            "paragraphs",
            sqlx::query_scalar("SELECT count(*) FROM paragraphs WHERE chapter_id = $1")
                .bind(whole.chapter_id)
                .fetch_one(pool)
                .await?,
        ),
        (
            "translations",
            sqlx::query_scalar("SELECT count(*) FROM translations WHERE paragraph_id = $1")
                .bind(whole.paragraph_id)
                .fetch_one(pool)
                .await?,
        ),
        (
            "context_snapshots",
            sqlx::query_scalar("SELECT count(*) FROM context_snapshots WHERE book_id = $1")
                .bind(whole.book_id)
                .fetch_one(pool)
                .await?,
        ),
        (
            "paragraph_embeddings",
            sqlx::query_scalar("SELECT count(*) FROM paragraph_embeddings WHERE paragraph_id = $1")
                .bind(whole.paragraph_id)
                .fetch_one(pool)
                .await?,
        ),
        (
            "paragraph_annotations",
            sqlx::query_scalar(
                "SELECT count(*) FROM paragraph_annotations WHERE paragraph_id = $1",
            )
            .bind(whole.paragraph_id)
            .fetch_one(pool)
            .await?,
        ),
        (
            "positions",
            sqlx::query_scalar("SELECT count(*) FROM positions WHERE book_id = $1")
                .bind(whole.book_id)
                .fetch_one(pool)
                .await?,
        ),
    ];
    for (table, count) in counts {
        if count != 1 {
            return Err(format!(
                "after the refused delete, {table} held {count} row(s), expected 1"
            )
            .into());
        }
    }

    sqlx::query("DELETE FROM paragraph_annotations WHERE paragraph_id = $1")
        .bind(whole.paragraph_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM paragraph_embeddings WHERE paragraph_id = $1")
        .bind(whole.paragraph_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM translations WHERE paragraph_id = $1")
        .bind(whole.paragraph_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM paragraphs WHERE id = $1")
        .bind(whole.paragraph_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM chapters WHERE id = $1")
        .bind(whole.chapter_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM context_snapshots WHERE book_id = $1")
        .bind(whole.book_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM positions WHERE book_id = $1")
        .bind(whole.book_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM books WHERE id = $1")
        .bind(whole.book_id)
        .execute(pool)
        .await?;

    let remaining: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM books WHERE id = $1)")
        .bind(whole.book_id)
        .fetch_one(pool)
        .await?;
    if remaining {
        return Err(
            "the book still existed after its content was removed in dependency order".into(),
        );
    }

    Ok(())
}

async fn assert_book_still_present(
    pool: &sqlx::PgPool,
    book_id: i64,
    case: &str,
) -> Result<(), Failed> {
    let remaining: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM books WHERE id = $1)")
        .bind(book_id)
        .fetch_one(pool)
        .await?;
    if !remaining {
        return Err(format!("the book disappeared despite the refused delete ({case})").into());
    }
    Ok(())
}

/// A book with a single chapter carrying no title holds its paragraphs, and
/// the paragraphs are reachable from the book through that chapter; a
/// paragraph attached to no chapter is refused.
async fn a_work_with_no_divisions_is_stored_as_one_untitled_chapter(
    harness: Arc<Harness>,
) -> Result<(), Failed> {
    let database = harness.create_database().await?;
    let pool = &database.pool;

    let book_id = schema::insert_book(pool).await?;
    let chapter_id: i64 = sqlx::query_scalar(
        "INSERT INTO chapters (book_id, idx, title) VALUES ($1, 0, NULL) RETURNING id",
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;
    let paragraph_id = schema::insert_paragraph(pool, chapter_id, 0).await?;

    let reachable_chapter_id: i64 =
        sqlx::query_scalar("SELECT chapter_id FROM paragraphs WHERE id = $1")
            .bind(paragraph_id)
            .fetch_one(pool)
            .await?;
    if reachable_chapter_id != chapter_id {
        return Err("the paragraph is not reachable from the book through its chapter".into());
    }

    let stored_title: Option<String> =
        sqlx::query_scalar("SELECT title FROM chapters WHERE id = $1")
            .bind(chapter_id)
            .fetch_one(pool)
            .await?;
    if stored_title.is_some() {
        return Err(
            format!("the untitled chapter's title round-tripped to {stored_title:?}").into(),
        );
    }

    let result = sqlx::query(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES (NULL, 1, 'p', '<p></p>', 't', $1)",
    )
    .bind(vec![0u8])
    .execute(pool)
    .await;
    schema::assert_sqlstate(result, "23502")
        .map_err(|err| format!("a paragraph attached to no chapter: {err}"))?;

    Ok(())
}
