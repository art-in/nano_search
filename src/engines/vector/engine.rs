use std::cell::RefCell;

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use ort::execution_providers::{CoreMLExecutionProvider, ExecutionProvider};
use rusqlite::Connection;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use zerocopy::IntoBytes;

use crate::model::doc::DocId;
use crate::model::engine::SearchEngine;

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

    fn create_on_disk(index_dir: impl AsRef<std::path::Path>) -> Result<Self>
    where
        Self: Sized,
    {
        if std::fs::exists(&index_dir)? {
            std::fs::remove_dir_all(&index_dir)?;
        }

        std::fs::create_dir(&index_dir)
            .context("index dir should be created")?;

        Self::init_vector_extension();

        let db = Connection::open(index_dir.as_ref().join("index"))?;
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
        let mut model = self.model.borrow_mut();
        let tx = self.db.transaction()?;
        let mut stmt =
            tx.prepare("INSERT INTO docs(rowid, embedding) VALUES (?, ?)")?;

        for doc in docs {
            let vectors = model.embed(vec![doc.text], None)?;
            let vector = &vectors[0];
            stmt.execute(rusqlite::params![doc.id, vector.as_bytes()])?;
        }

        drop(stmt);
        tx.commit()?;
        Ok(())
    }

    fn search(&self, query: &str, limit: u64) -> Result<Vec<DocId>> {
        let mut model = self.model.borrow_mut();

        let vectors = model.embed(vec![query], None)?;
        let vector = &vectors[0];

        let result: Vec<u64> = self
            .db
            .prepare_cached(
                r"
                SELECT rowid
                FROM docs
                WHERE embedding MATCH ?1
                ORDER BY distance
                LIMIT ?2
                ",
            )?
            .query_map((vector.as_bytes(), limit), |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

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
        db.execute(
            "CREATE VIRTUAL TABLE docs USING vec0(embedding float[384])",
            [],
        )?;
        Ok(Self { db, model })
    }

    fn create_model() -> Result<RefCell<TextEmbedding>> {
        // trying to speedup inference by using CoreML execution provider on
        // macbook pro m2. by default it greatly slows down inference (x7).
        // tried togging all the available options, and noticed no
        // effect, except with_static_input_shapes(true), which makes
        // inference as fast as with default CPU provider.
        //
        // provider is not supported within docker container, have to run on
        // macos directly, otherwise default CPU provider will be used.
        //
        // keeping it for now, as it may improve perf with other models.
        //
        // CoreML options doc:
        // https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html
        let corelm_provider = CoreMLExecutionProvider::default()
            .with_subgraphs(true) // no effect on performance
            .with_static_input_shapes(true); // x7 speedup

        println!(
            "CoreLM enabled: {}",
            corelm_provider.supported_by_platform()
        );

        // setup ONNX runtime execution providers. if none of them is supported
        // by current platform, then CPU provider will be used as a fallback
        let providers = vec![corelm_provider.build()];

        Ok(RefCell::new(TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_execution_providers(providers)
                .with_show_download_progress(true),
        )?))
    }
}
