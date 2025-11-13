use std::cell::RefCell;

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, ScalarQuantizationBuilder,
    SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder,
    VectorParamsBuilder, point_id,
};

use crate::model::doc::DocId;
use crate::model::engine::SearchEngine;
use crate::utils::HF_CACHE_DIR;

const VECTOR_SIZE: u64 = 384;

pub struct QdrantSearchEngine {
    runtime: tokio::runtime::Runtime,
    client: Qdrant,
    model: RefCell<Box<TextEmbedding>>,
}

const COLLECTION_NAME: &str = "docs";

impl SearchEngine for QdrantSearchEngine {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "qdrant"
    }

    fn get_name(&self) -> &'static str {
        Self::name()
    }

    fn create_in_memory() -> Result<Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn create_on_disk(_index_dir: impl AsRef<std::path::Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let engine = Self::open_from_disk(_index_dir)?;

        engine
            .runtime
            .block_on(engine.client.delete_collection(COLLECTION_NAME))?;

        engine.runtime.block_on(
            engine.client.create_collection(
                CreateCollectionBuilder::new(COLLECTION_NAME)
                    .vectors_config(VectorParamsBuilder::new(
                        VECTOR_SIZE,
                        Distance::Cosine,
                    ))
                    .quantization_config(
                        // disable quantization for better search quality
                        ScalarQuantizationBuilder::default().quantile(1.0),
                    ),
            ),
        )?;

        Ok(engine)
    }

    fn open_from_disk(_index_dir: impl AsRef<std::path::Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let client =
            Qdrant::from_url("http://host.docker.internal:6334").build()?;

        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
                .with_cache_dir(HF_CACHE_DIR.into()),
        )?;

        Ok(Self {
            runtime,
            client,
            model: RefCell::new(Box::new(model)),
        })
    }

    fn index_docs(
        &mut self,
        docs: &mut dyn Iterator<Item = crate::model::doc::Doc>,
    ) -> Result<()> {
        let mut points = Vec::new();

        for doc in docs {
            let id = point_id::PointIdOptions::Num(doc.id);
            let vector = self.model.get_mut().embed(vec![doc.text], None)?;
            let vector = vector[0].clone();
            let payload: qdrant_client::Payload =
                serde_json::json!({}).try_into()?;

            points.push(PointStruct::new(id, vector, payload));
        }

        self.runtime.block_on(self.client.upsert_points(
            UpsertPointsBuilder::new(COLLECTION_NAME, points),
        ))?;

        Ok(())
    }

    fn search(&self, query: &str, limit: u64) -> Result<Vec<DocId>> {
        let mut model = self.model.borrow_mut();
        let search_vector = model.embed(vec![query], None)?;
        let search_vector = search_vector[0].clone();
        let search_result = self.runtime.block_on(
            self.client.search_points(
                SearchPointsBuilder::new(COLLECTION_NAME, search_vector, limit)
                    .params(SearchParamsBuilder::default().exact(true))
                    .with_payload(false),
            ),
        )?;

        let mut res = Vec::with_capacity(search_result.result.len());

        for point in search_result.result {
            if let Some(id) = point.id
                && let Some(id) = id.point_id_options
                && let point_id::PointIdOptions::Num(id) = id
            {
                res.push(id);
            }
        }

        Ok(res)
    }
}
