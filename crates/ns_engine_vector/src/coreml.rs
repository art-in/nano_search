use fastembed::ExecutionProviderDispatch;
use ort::execution_providers::{CoreMLExecutionProvider, ExecutionProvider};
use tracing::info;

pub fn init_coreml_provider() -> ExecutionProviderDispatch {
    // Trying to speedup inference by using CoreML provider on macbook pro m2
    //
    // CoreML able to execute only part of model or nothing at all:
    // - with MiniLM models only 231/323 model nodes supported, rest is on CPU.
    //   and it is 7x slower than CPU-only, probably due to lots of CPU-GPU
    //   memory transfers
    // - with BGE models 0 nodes are supported, so it is CPU-only
    //
    // Debug log says that "CoreML does not support input dim > 16384", which
    // looks like compatibility issue between model and CoreML/Apple hardware
    //
    // Debug: `RUST_LOG="ort=debug" cargo run`
    //
    // Related issue: https://github.com/microsoft/onnxruntime/issues/19543
    //
    // Keeping it for now, as it may work better with other models
    let corelm_provider = CoreMLExecutionProvider::default();

    info!(
        "CoreLM enabled: {}",
        corelm_provider.supported_by_platform()
    );

    corelm_provider.build()
}
