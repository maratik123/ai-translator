//! Fixtures shared by the schema trials, and the function that assembles
//! this module's whole trial list for the database-backed target's `main`
//! to extend its own list with.
//!
//! One submodule reads the catalogue; the other drives the database.

mod behaviour;
mod shape;

use std::sync::Arc;

use libtest_mimic::{Failed, Trial};
use sqlx::PgPool;
use tokio::runtime::Handle;

use crate::support::Harness;

/// The dimension the embedding column declares.
pub const EMBEDDING_DIMENSION: usize = 1024;

/// Builds this module's whole trial list.
pub fn trials(harness: &Arc<Harness>, handle: &Handle) -> Vec<Trial> {
    let mut trials = shape::trials(harness, handle);
    trials.extend(behaviour::trials(harness, handle));
    trials
}

/// A bracketed vector literal of `dimension` zero components, in the form
/// `"[0,0,…]"` `pgvector`'s input function accepts.
pub fn zero_vector_literal(dimension: usize) -> String {
    let values = vec!["0"; dimension].join(",");
    format!("[{values}]")
}

/// The identifiers of a book, one chapter under it, and one paragraph under
/// that chapter — the smallest fixture several trials need.
pub struct BookChapterParagraph {
    pub book_id: i64,
    pub chapter_id: i64,
    pub paragraph_id: i64,
}

/// Inserts a book and returns its identifier.
pub async fn insert_book(pool: &PgPool) -> Result<i64, Failed> {
    let book_id: i64 = sqlx::query_scalar(
        "INSERT INTO books (title, lang_src, lang_dst, format, added_at) \
         VALUES ('a book', 'en', 'ru', 'epub', now()) RETURNING id",
    )
    .fetch_one(pool)
    .await?;
    Ok(book_id)
}

/// Inserts a chapter under `book_id` at position `idx` and returns its
/// identifier.
pub async fn insert_chapter(pool: &PgPool, book_id: i64, idx: i32) -> Result<i64, Failed> {
    let chapter_id: i64 =
        sqlx::query_scalar("INSERT INTO chapters (book_id, idx) VALUES ($1, $2) RETURNING id")
            .bind(book_id)
            .bind(idx)
            .fetch_one(pool)
            .await?;
    Ok(chapter_id)
}

/// Inserts a paragraph under `chapter_id` at position `idx` and returns its
/// identifier.
pub async fn insert_paragraph(pool: &PgPool, chapter_id: i64, idx: i32) -> Result<i64, Failed> {
    let paragraph_id: i64 = sqlx::query_scalar(
        "INSERT INTO paragraphs (chapter_id, idx, kind, html, text, stable_hash) \
         VALUES ($1, $2, 'p', '<p></p>', 'text', $3) RETURNING id",
    )
    .bind(chapter_id)
    .bind(idx)
    .bind(vec![0u8, 1, 2])
    .fetch_one(pool)
    .await?;
    Ok(paragraph_id)
}

/// Inserts a book, a chapter under it and a paragraph under that chapter.
pub async fn insert_book_chapter_paragraph(pool: &PgPool) -> Result<BookChapterParagraph, Failed> {
    let book_id = insert_book(pool).await?;
    let chapter_id = insert_chapter(pool, book_id, 0).await?;
    let paragraph_id = insert_paragraph(pool, chapter_id, 0).await?;
    Ok(BookChapterParagraph {
        book_id,
        chapter_id,
        paragraph_id,
    })
}

/// A book with a row in every referencing table: a chapter, a paragraph
/// under it, a translation, a context snapshot, an embedding, an annotation
/// and a reading position.
pub struct WholeBook {
    pub book_id: i64,
    pub chapter_id: i64,
    pub paragraph_id: i64,
}

/// Builds a [`WholeBook`].
pub async fn insert_whole_book(pool: &PgPool) -> Result<WholeBook, Failed> {
    let BookChapterParagraph {
        book_id,
        chapter_id,
        paragraph_id,
    } = insert_book_chapter_paragraph(pool).await?;

    sqlx::query(
        "INSERT INTO translations \
         (paragraph_id, cache_key, model, prompt_version, context_version, text, created_at) \
         VALUES ($1, $2, 'm', 1, 1, 't', now())",
    )
    .bind(paragraph_id)
    .bind(vec![1u8])
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO context_snapshots \
         (book_id, version, upto_paragraph_id, summary, glossary, model, created_at) \
         VALUES ($1, 1, $2, 's', $3::jsonb, 'm', now())",
    )
    .bind(book_id)
    .bind(paragraph_id)
    .bind("{}")
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO paragraph_embeddings (paragraph_id, model, embedding) \
         VALUES ($1, 'm', $2::vector)",
    )
    .bind(paragraph_id)
    .bind(zero_vector_literal(EMBEDDING_DIMENSION))
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO paragraph_annotations (paragraph_id, context_version, annotated_text, uncertain) \
         VALUES ($1, 1, 'a', false)",
    )
    .bind(paragraph_id)
    .execute(pool)
    .await?;

    sqlx::query("INSERT INTO positions (book_id, paragraph_id, updated_at) VALUES ($1, $2, now())")
        .bind(book_id)
        .bind(paragraph_id)
        .execute(pool)
        .await?;

    Ok(WholeBook {
        book_id,
        chapter_id,
        paragraph_id,
    })
}

/// The SQLSTATE a database error carries, when it carries one at all.
pub fn sqlstate_of(err: &sqlx::Error) -> Option<String> {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().map(|code| code.into_owned()),
        _ => None,
    }
}

/// Asserts that a statement's result is an error carrying exactly `expected`
/// as its SQLSTATE — never merely "an error", because a case that asserts
/// only failure passes on the wrong refusal.
pub fn assert_sqlstate<T>(result: Result<T, sqlx::Error>, expected: &str) -> Result<(), String> {
    match result {
        Ok(_) => Err(format!(
            "expected SQLSTATE {expected}, the statement succeeded"
        )),
        Err(err) => {
            let code = sqlstate_of(&err);
            if code.as_deref() == Some(expected) {
                Ok(())
            } else {
                Err(format!(
                    "expected SQLSTATE {expected}, got {code:?} ({err})"
                ))
            }
        }
    }
}
