use std::cell::RefCell;
use std::time::Instant;

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use itertools::Itertools;
use rusqlite::Connection;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use tracing::debug;
use zerocopy::IntoBytes;

use crate::model::doc::{Doc, DocId};
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};
use crate::utils::HF_CACHE_DIR;

pub struct VectorSearchEngine {
    db: Connection,

    // wrapping model into RefCell, since model.embed() requires mutable
    // model reference for some reason
    model: RefCell<TextEmbedding>,
}

impl SearchEngine for VectorSearchEngine {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "vector"
    }

    fn get_name(&self) -> &'static str {
        Self::name()
    }

    fn create_in_memory() -> Result<Self>
    where
        Self: Sized,
    {
        Self::init_vector_extension();

        let db = Connection::open_in_memory()?;
        Self::create_with_connection(db)
    }

    fn create_on_disk(opts: CreateOnDiskOptions) -> Result<Self>
    where
        Self: Sized,
    {
        if std::fs::exists(&opts.index_dir)? {
            std::fs::remove_dir_all(&opts.index_dir)?;
        }

        std::fs::create_dir(&opts.index_dir)
            .context("index dir should be created")?;

        Self::init_vector_extension();

        let db = Connection::open(opts.index_dir.join("index"))?;
        Self::create_with_connection(db)
    }

    fn open_from_disk(index_dir: impl AsRef<std::path::Path>) -> Result<Self>
    where
        Self: Sized,
    {
        Self::init_vector_extension();

        let db = Connection::open(index_dir.as_ref().join("index"))?;
        let model = Self::create_model()?;

        Ok(Self { db, model })
    }

    fn index_docs(
        &mut self,
        docs: &mut dyn Iterator<Item = crate::model::doc::Doc>,
    ) -> Result<()> {
        let tx = self.db.transaction()?;
        let mut stmt =
            tx.prepare("INSERT INTO docs(docid, embedding) VALUES (?, ?)")?;

        // experimenting with batched embedding. noticed slow down compared to
        // sequential embedding. should work better with hardware acceleration
        const EMBED_DOCS_BATCH_SIZE: usize = 1;

        for docs_batch in &docs.chunks(EMBED_DOCS_BATCH_SIZE) {
            let docs_batch = docs_batch.collect::<Vec<Doc>>();

            let texts_batch = docs_batch
                .iter()
                .map(|d| d.text.as_str())
                .collect::<Vec<&str>>();

            let vectors = embed(&self.model, texts_batch)?;

            for (idx, doc) in docs_batch.iter().enumerate() {
                stmt.execute(rusqlite::params![
                    &doc.id.to_string(),
                    vectors[idx].as_bytes()
                ])?;
            }
        }

        drop(stmt);
        tx.commit()?;
        Ok(())
    }

    fn search(&self, query: &str, limit: u64) -> Result<Vec<DocId>> {
        let vectors = embed(&self.model, vec![query])?;
        let vector = &vectors[0];

        let result: Vec<u64> = self
            .db
            .prepare_cached(
                r"
                SELECT docid
                FROM docs
                WHERE embedding MATCH ?1
                ORDER BY distance
                LIMIT ?2
                ",
            )?
            .query_map((vector.as_bytes(), limit), |row| {
                let docid: String = row.get(0)?;
                Ok(docid)
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|docid_str| {
                docid_str.parse::<u64>().context(format!(
                    "numeric docid should be parsed from {}",
                    docid_str
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(result)
    }
}

impl VectorSearchEngine {
    #[allow(clippy::missing_transmute_annotations)]
    fn init_vector_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_vec_init as *const (),
            )));
        }
    }

    fn create_with_connection(db: Connection) -> Result<Self> {
        let model = Self::create_model()?;
        // sqlite does not support u64 type, so store docids as text
        db.execute(
            "CREATE VIRTUAL TABLE docs USING vec0(docid TEXT PRIMARY KEY, \
             embedding float[384])",
            [],
        )?;
        Ok(Self { db, model })
    }

    fn create_model() -> Result<RefCell<TextEmbedding>> {
        // setup ONNX runtime execution providers. if none of them is supported
        // by current platform, then CPU provider will be used as a fallback
        let providers = vec![
            #[cfg(feature = "coreml")]
            super::coreml::init_coreml_provider(),
        ];

        Ok(RefCell::new(TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_execution_providers(providers)
                .with_show_download_progress(true)
                .with_cache_dir(HF_CACHE_DIR.into()),
        )?))
    }
}

pub fn embed(
    model: &RefCell<TextEmbedding>,
    texts: Vec<&str>,
) -> Result<Vec<Vec<f32>>> {
    let mut model = model.borrow_mut();

    let texts_count = texts.len();
    let texts_size_kb =
        texts.iter().fold(0.0, |acc, e| acc + e.len() as f64) / 1000.0;

    let now = Instant::now();

    let vectors = model.embed(texts, None)?;

    let elapsed = now.elapsed();
    debug!(
        "embed: {:>4} ms/text, {:>6.0} ms/1KB, {:>6.2} KB/sec",
        elapsed.as_millis() as usize / texts_count,
        elapsed.as_millis() as f64 / texts_size_kb,
        texts_size_kb / elapsed.as_secs_f64()
    );

    Ok(vectors)
}
