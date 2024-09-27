use std::{collections::HashMap, net::SocketAddr};

use anyhow::anyhow;
use futures_util::StreamExt;
use moly_protocol::{
    open_ai::{
        ChatResponse, ChatResponseChunkData, ChatResponseData, ChunkChoiceData, MessageData, Role,
        StopReason,
    },
    protocol::LoadModelOptions,
};
use wasmedge_sdk::{wasi::WasiModule, Module, Store, Vm};

use crate::store::download_files::DownloadedFile;

use super::BackendModel;

// From https://github.com/L-jasmine/LlamaEdge/tree/feat/support_unload_and_exit
// A repo that fork from LlamaEdge/LlamaEdge for support unload model and exit
static WASM: &[u8] = include_bytes!("../../wasm/llama-api-server.wasm");

/// Use server which is OpenAI compatible
pub struct LLamaEdgeApiServer {
    id: String,
    listen_addr: SocketAddr,
    load_model_options: LoadModelOptions,
    wasm_module: Module,
    embedding: Option<(std::path::PathBuf, u64)>,
    running_controller: tokio::sync::broadcast::Sender<()>,
    #[allow(dead_code)]
    model_thread: std::thread::JoinHandle<()>,
}

fn create_wasi(
    listen_addr: SocketAddr,
    file: &DownloadedFile,
    load_model: &LoadModelOptions,
    embedding: Option<(std::path::PathBuf, u64)>,
) -> wasmedge_sdk::WasmEdgeResult<WasiModule> {
    // use model metadata context size
    let ctx_size_str = if let Some(n_ctx) = load_model.n_ctx {
        format!("{}", n_ctx)
    } else {
        format!("{}", file.context_size.min(8 * 1024))
    };

    let ctx_size = if let Some((_, embedding_ctx)) = embedding {
        Some(format!("{},{}", ctx_size_str, embedding_ctx))
    } else {
        Some(ctx_size_str)
    };

    let n_gpu_layers = match load_model.gpu_layers {
        moly_protocol::protocol::GPULayers::Specific(n) => Some(n.to_string()),
        moly_protocol::protocol::GPULayers::Max => None,
    };

    // Set n_batch to a fixed value of 128.
    let batch_size = if let Some(n_batch) = load_model.n_batch {
        n_batch
    } else {
        128
    };

    let batch_size = if let Some((_, embedding_ctx)) = embedding {
        Some(format!("{},{}", batch_size, embedding_ctx))
    } else {
        Some(format!("{}", batch_size))
    };

    let mut prompt_template = load_model.prompt_template.clone();
    if prompt_template.is_none() && !file.prompt_template.is_empty() {
        prompt_template = Some(file.prompt_template.clone());
    }

    if embedding.is_some() {
        if let Some(ref mut prompt_template) = prompt_template {
            prompt_template.push_str(",");
            prompt_template.push_str("embedding");
        }
    }

    let reverse_prompt = if file.reverse_prompt.is_empty() {
        None
    } else {
        Some(file.reverse_prompt.clone())
    };

    let listen_addr = Some(format!("{listen_addr}"));

    let mut module_alias = "moly-chat".to_string();
    if embedding.is_some() {
        module_alias.push_str(",");
        module_alias.push_str("moly-embedding");
    }
    let mut args = vec![
        "llama-api-server",
        "-a",
        module_alias.as_str(),
        "-m",
        module_alias.as_str(),
    ];

    macro_rules! add_args {
        ($flag:expr, $value:expr) => {
            if let Some(ref value) = $value {
                args.push($flag);
                args.push(value.as_str());
            }
        };
    }

    add_args!("-c", ctx_size);
    add_args!("-g", n_gpu_layers);
    add_args!("-b", batch_size);
    add_args!("-p", prompt_template);
    add_args!("-r", reverse_prompt);
    add_args!("--socket-addr", listen_addr);

    WasiModule::create(Some(args), None, None)
}

pub fn run_wasm_by_downloaded_file(
    listen_addr: SocketAddr,
    wasm_module: Module,
    file: DownloadedFile,
    load_model: LoadModelOptions,
    embedding: Option<(std::path::PathBuf, u64)>,
) {
    use wasmedge_sdk::AsInstance;

    let mut instances = HashMap::new();

    let mut wasi = create_wasi(listen_addr, &file, &load_model, embedding).unwrap();
    instances.insert(wasi.name().to_string(), wasi.as_mut());

    let mut wasi_nn = wasmedge_sdk::plugin::PluginManager::load_plugin_wasi_nn().unwrap();
    instances.insert(wasi_nn.name().unwrap(), &mut wasi_nn);

    let store = Store::new(None, instances).unwrap();
    let mut vm = Vm::new(store);
    vm.register_module(None, wasm_module.clone()).unwrap();

    let _ = vm.run_func(None, "_start", []);

    log::debug!("wasm exit");
}

fn stop_chunk(reason: StopReason) -> ChatResponseChunkData {
    ChatResponseChunkData {
        id: String::new(),
        choices: vec![ChunkChoiceData {
            finish_reason: Some(reason),
            index: 0,
            delta: MessageData {
                content: String::new(),
                role: Role::Assistant,
            },
            logprobs: None,
        }],
        created: 0,
        model: String::new(),
        system_fingerprint: String::new(),
        object: "chat.completion.chunk".to_string(),
    }
}

impl BackendModel for LLamaEdgeApiServer {
    fn new_or_reload(
        async_rt: &tokio::runtime::Runtime,
        old_model: Option<Self>,
        file: crate::store::download_files::DownloadedFile,
        options: moly_protocol::protocol::LoadModelOptions,
        tx: std::sync::mpsc::Sender<anyhow::Result<moly_protocol::protocol::LoadModelResponse>>,
        embedding: Option<(std::path::PathBuf, u64)>,
    ) -> Self {
        let load_model_options = options.clone();
        let mut need_reload = true;
        let (wasm_module, listen_addr) = if let Some(old_model) = &old_model {
            if old_model.id == file.id.as_str()
                && old_model.load_model_options.n_ctx == options.n_ctx
                && old_model.load_model_options.n_batch == options.n_batch
                && old_model.embedding == embedding
            {
                need_reload = false;
            }
            (old_model.wasm_module.clone(), old_model.listen_addr)
        } else {
            let addr = std::env::var("MOLY_API_SERVER_ADDR").unwrap_or("localhost:0".to_string());
            let new_addr = std::net::TcpListener::bind(&addr)
                .unwrap()
                .local_addr()
                .unwrap();

            (Module::from_bytes(None, WASM).unwrap(), new_addr)
        };

        if !need_reload {
            let _ = tx.send(Ok(moly_protocol::protocol::LoadModelResponse::Completed(
                moly_protocol::protocol::LoadedModelInfo {
                    file_id: file.id.to_string(),
                    model_id: file.model_id,
                    information: "".to_string(),
                    listen_port: listen_addr.port(),
                },
            )));
            return old_model.unwrap();
        }

        if let Some(old_model) = old_model {
            old_model.stop(async_rt);
        }

        let wasm_module_ = wasm_module.clone();

        let file_id = file.id.to_string();

        let listen_port = listen_addr.port();
        let url = format!("http://localhost:{}/echo", listen_port);

        let file_ = file.clone();

        let embedding_ = embedding.clone();

        let model_thread = std::thread::spawn(move || {
            run_wasm_by_downloaded_file(listen_addr, wasm_module_, file, options, embedding_)
        });

        async_rt.spawn(async move {
            let mut test_server = false;
            for _i in 0..600 {
                let r = reqwest::ClientBuilder::new()
                    .no_proxy()
                    .build()
                    .unwrap()
                    .get(&url)
                    .send()
                    .await;
                if let Ok(resp) = r {
                    if resp.status().is_success() {
                        test_server = true;
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            if test_server {
                let _ = tx.send(Ok(moly_protocol::protocol::LoadModelResponse::Completed(
                    moly_protocol::protocol::LoadedModelInfo {
                        file_id: file_.id.to_string(),
                        model_id: file_.model_id,
                        information: "".to_string(),
                        listen_port,
                    },
                )));
            } else {
                let _ = tx.send(Err(anyhow!("Failed to start the model")));
            }
        });

        let running_controller = tokio::sync::broadcast::channel(1).0;

        let new_model = Self {
            id: file_id,
            wasm_module,
            embedding,
            listen_addr,
            running_controller,
            model_thread,
            load_model_options,
        };

        new_model
    }

    fn chat(
        &self,
        async_rt: &tokio::runtime::Runtime,
        mut data: moly_protocol::open_ai::ChatRequestData,
        tx: std::sync::mpsc::Sender<anyhow::Result<ChatResponse>>,
    ) -> bool {
        let is_stream = data.stream.unwrap_or(false);
        let url = format!(
            "http://localhost:{}/v1/chat/completions",
            self.listen_addr.port()
        );
        let mut cancel = self.running_controller.subscribe();

        data.model = "moly-chat".to_string();

        async_rt.spawn(async move {
            let request_body = serde_json::to_string(&data).unwrap();
            let resp = reqwest::ClientBuilder::new()
                .no_proxy()
                .build()
                .unwrap()
                .post(url)
                .body(request_body)
                .send()
                .await
                .map_err(|e| anyhow!(e));

            match resp {
                Ok(resp) => {
                    if is_stream {
                        let mut stream = resp.bytes_stream();

                        while let Some(chunk) = tokio::select! {
                            chunk = stream.next() => chunk,
                            _ = cancel.recv() => None,
                        } {
                            match chunk {
                                Ok(chunk) => {
                                    if chunk.starts_with(b"data: [DONE]") {
                                        break;
                                    }
                                    let resp: Result<ChatResponseChunkData, anyhow::Error> =
                                        serde_json::from_slice(&chunk[5..]).map_err(|e| anyhow!(e));
                                    let _ = tx.send(resp.map(ChatResponse::ChatResponseChunk));
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(anyhow!(e)));
                                    return;
                                }
                            }
                        }

                        let _ = tx.send(Ok(ChatResponse::ChatResponseChunk(stop_chunk(
                            StopReason::Stop,
                        ))));
                    } else {
                        let resp: Result<ChatResponseData, anyhow::Error> =
                            resp.json().await.map_err(|e| anyhow!(e));
                        let _ = tx.send(resp.map(ChatResponse::ChatFinalResponseData));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    let _ = tx.send(Ok(ChatResponse::ChatResponseChunk(stop_chunk(
                        StopReason::Stop,
                    ))));
                }
            }
        });

        true
    }

    fn stop_chat(&self, _async_rt: &tokio::runtime::Runtime) {
        let _ = self.running_controller.send(());
    }

    fn stop(self, _async_rt: &tokio::runtime::Runtime) {
        let url = format!("http://localhost:{}/admin/exit", self.listen_addr.port());
        let _ = reqwest::blocking::ClientBuilder::new()
            .no_proxy()
            .build()
            .unwrap()
            .get(url)
            .send();
        let _ = self.model_thread.join();
    }
}
