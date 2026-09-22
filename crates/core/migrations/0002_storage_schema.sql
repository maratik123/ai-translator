CREATE TABLE books (
    id bigserial PRIMARY KEY,
    title text NOT NULL,
    author text,
    lang_src text NOT NULL,
    lang_dst text NOT NULL,
    format text NOT NULL,
    cover_path text,
    added_at timestamptz NOT NULL
);

CREATE TABLE chapters (
    id bigserial PRIMARY KEY,
    book_id bigint NOT NULL REFERENCES books (id) ON DELETE RESTRICT,
    idx int NOT NULL,
    title text,
    css text
);

CREATE TABLE paragraphs (
    id bigserial PRIMARY KEY,
    chapter_id bigint NOT NULL REFERENCES chapters (id) ON DELETE RESTRICT,
    idx int NOT NULL,
    kind text NOT NULL,
    html text NOT NULL,
    text text NOT NULL,
    stable_hash bytea NOT NULL
);

CREATE TABLE translations (
    paragraph_id bigint NOT NULL REFERENCES paragraphs (id) ON DELETE RESTRICT,
    cache_key bytea NOT NULL,
    model text NOT NULL,
    prompt_version int NOT NULL,
    context_version int NOT NULL,
    text text NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (paragraph_id, cache_key, context_version)
);

CREATE TABLE context_snapshots (
    book_id bigint NOT NULL REFERENCES books (id) ON DELETE RESTRICT,
    version int NOT NULL,
    upto_paragraph_id bigint NOT NULL,
    summary text NOT NULL,
    glossary jsonb NOT NULL,
    model text NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (book_id, version)
);

CREATE TABLE paragraph_embeddings (
    paragraph_id bigint PRIMARY KEY REFERENCES paragraphs (id) ON DELETE RESTRICT,
    model text NOT NULL,
    embedding vector(1024) NOT NULL
);

CREATE TABLE paragraph_annotations (
    paragraph_id bigint NOT NULL REFERENCES paragraphs (id) ON DELETE RESTRICT,
    context_version int NOT NULL,
    annotated_text text NOT NULL,
    uncertain boolean NOT NULL,
    PRIMARY KEY (paragraph_id, context_version)
);

CREATE TABLE positions (
    book_id bigint PRIMARY KEY REFERENCES books (id) ON DELETE RESTRICT,
    paragraph_id bigint NOT NULL,
    updated_at timestamptz NOT NULL
);

CREATE TABLE settings (
    key text PRIMARY KEY,
    value jsonb NOT NULL
);

CREATE UNIQUE INDEX chapters_book_id_idx_uniq ON chapters (book_id, idx);
CREATE UNIQUE INDEX paragraphs_chapter_id_idx_uniq ON paragraphs (chapter_id, idx);
CREATE INDEX translations_paragraph_id_idx ON translations (paragraph_id);
CREATE INDEX context_snapshots_book_id_upto_paragraph_id_idx ON context_snapshots (book_id, upto_paragraph_id);
