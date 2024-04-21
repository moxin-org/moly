use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use chrono::Utc;
use moxin_protocol::{
    data::{DownloadedFile, FileID, Model, PendingDownload},
    open_ai::{ChatRequestData, ChatResponse},
    protocol::{
        Command, FileDownloadResponse, LoadModelOptions, LoadModelResponse, LocalServerConfig,
        LocalServerResponse,
    },
};
use wasmedge_sdk::Module;

use crate::store::{self, pending_downloads::PendingDownloads, RemoteModel};

mod chat_ui {

    #[derive(Debug)]
    pub enum TokenError {
        EndOfSequence = 1,
        ContextFull,
        PromptTooLong,
        TooLarge,
        InvalidEncoding,
        Other,
    }

    impl Into<StopReason> for TokenError {
        fn into(self) -> StopReason {
            match self {
                TokenError::EndOfSequence => StopReason::Stop,
                TokenError::ContextFull => StopReason::Length,
                TokenError::PromptTooLong => StopReason::Length,
                TokenError::TooLarge => StopReason::Length,
                TokenError::InvalidEncoding => StopReason::Stop,
                TokenError::Other => StopReason::Stop,
            }
        }
    }

    use std::{
        collections::HashMap,
        io::Read,
        path::Path,
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc::{Receiver, Sender},
            Arc,
        },
        thread::JoinHandle,
    };

    use moxin_protocol::{
        open_ai::{
            ChatRequestData, ChatResponse, ChatResponseChunkData, ChatResponseData, ChoiceData,
            ChunkChoiceData, MessageData, Role, StopReason, UsageData,
        },
        protocol::{LoadModelOptions, LoadModelResponse, LoadedModelInfo},
    };
    use wasmedge_sdk::{
        error::{CoreError, CoreExecutionError},
        wasi::WasiModule,
        CallingFrame, ImportObject, Instance, Module, Store, Vm, WasmValue,
    };

    use crate::store::download_files::DownloadedFile;

    #[derive(Debug)]
    pub struct ChatBotUi {
        pub current_req: std::io::Cursor<Vec<u8>>,
        pub request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
        request_id: uuid::Uuid,
        chat_completion_message: Option<Vec<u8>>,
        pub token_tx: Option<Sender<anyhow::Result<ChatResponse>>>,
        running_controller: Arc<AtomicBool>,
        pub load_model_state: Option<(
            DownloadedFile,
            LoadModelOptions,
            Sender<anyhow::Result<LoadModelResponse>>,
        )>,
    }

    impl ChatBotUi {
        pub fn new(
            request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
            running_controller: Arc<AtomicBool>,
            file: DownloadedFile,
            load_model: LoadModelOptions,
            tx: Sender<anyhow::Result<LoadModelResponse>>,
        ) -> Self {
            Self {
                request_rx,
                request_id: uuid::Uuid::new_v4(),
                token_tx: None,
                running_controller,
                current_req: std::io::Cursor::new(vec![]),
                load_model_state: Some((file, load_model, tx)),
                chat_completion_message: None,
            }
        }

        fn init_request(&mut self) -> Result<(), ()> {
            if let Ok((req, tx)) = self.request_rx.recv() {
                // Init current_req
                if !req.stream.unwrap_or_default() {
                    self.chat_completion_message = Some(Vec::with_capacity(
                        (req.max_tokens.unwrap_or(512) * 8) as usize,
                    ))
                }
                *self.current_req.get_mut() = serde_json::to_vec(&req).unwrap();
                self.current_req.set_position(0);
                self.request_id = uuid::Uuid::new_v4();
                self.token_tx = Some(tx);
                self.running_controller.store(true, Ordering::Release);
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn read_data(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.current_req.read(buf)?;
            if n == 0 {
                self.current_req.get_mut().clear();
                self.current_req.set_position(0);
            }
            Ok(n)
        }

        fn send_completion_output(
            token_tx: &mut Sender<anyhow::Result<ChatResponse>>,
            id: String,
            stop_reason: StopReason,
            chat_completion_message: &mut Option<Vec<u8>>,
        ) -> bool {
            if let Some(chat_completion_message) = chat_completion_message.take() {
                let _ = token_tx.send(Ok(ChatResponse::ChatFinalResponseData(ChatResponseData {
                    id,
                    choices: vec![ChoiceData {
                        finish_reason: stop_reason,
                        index: 0,
                        message: MessageData {
                            content: String::from_utf8_lossy(&chat_completion_message).to_string(),
                            role: Role::Assistant,
                        },
                        logprobs: None,
                    }],
                    created: 0,
                    model: String::new(),
                    system_fingerprint: String::new(),
                    usage: UsageData {
                        completion_tokens: 0,
                        prompt_tokens: 0,
                        total_tokens: 0,
                    },
                    object: "chat.completion".to_string(),
                })));
            } else {
                let _ = token_tx.send(Ok(ChatResponse::ChatResponseChunk(ChatResponseChunkData {
                    id: String::new(),
                    choices: vec![ChunkChoiceData {
                        finish_reason: Some(stop_reason),
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
                })));
            };
            true
        }

        fn send_streamed_output(
            token_tx: &mut Sender<anyhow::Result<ChatResponse>>,
            id: String,
            token: &[u8],
        ) -> bool {
            let _ = token_tx.send(Ok(ChatResponse::ChatResponseChunk(ChatResponseChunkData {
                id,
                choices: vec![ChunkChoiceData {
                    finish_reason: None,
                    index: 0,
                    delta: MessageData {
                        content: String::from_utf8_lossy(token).to_string(),
                        role: Role::Assistant,
                    },
                    logprobs: None,
                }],
                created: 0,
                model: String::new(),
                system_fingerprint: String::new(),
                object: "chat.completion.chunk".to_string(),
            })));
            true
        }

        fn send_output(&mut self, output: Result<&[u8], TokenError>) -> bool {
            let id = self.request_id.to_string();
            match (
                output,
                &mut self.chat_completion_message,
                &mut self.token_tx,
            ) {
                (Ok(token), Some(chat_completion_message), Some(_tx)) => {
                    chat_completion_message.extend_from_slice(token);
                    true
                }
                (Ok(token), None, Some(tx)) => Self::send_streamed_output(tx, id, token),
                (Err(token_error), chat_completion_message, Some(tx)) => {
                    Self::send_completion_output(
                        tx,
                        id,
                        token_error.into(),
                        chat_completion_message,
                    )
                }
                (_, _, None) => false,
            }
        }
    }

    fn get_input(
        data: &mut ChatBotUi,
        _inst: &mut Instance,
        frame: &mut CallingFrame,
        args: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        let mem = frame
            .memory_mut(0)
            .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

        if let Some([buf_ptr, buf_size]) = args.get(0..2) {
            let buf_ptr = buf_ptr.to_i32() as usize;
            let buf_size = buf_size.to_i32() as usize;

            let buf = mem
                .mut_slice::<u8>(buf_ptr, buf_size)
                .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

            if data.current_req.get_ref().is_empty() {
                if let Some((file, _, tx)) = data.load_model_state.take() {
                    let file_id = file.id.as_ref().clone();
                    let model_id = file.model_id;
                    let _ = tx.send(Ok(LoadModelResponse::Completed(LoadedModelInfo {
                        file_id,
                        model_id,
                        information: String::new(),
                    })));
                }

                data.init_request().or(Err(CoreError::Common(
                    wasmedge_sdk::error::CoreCommonError::Interrupted,
                )))?;
            }

            let n = data.read_data(buf).unwrap();

            Ok(vec![WasmValue::from_i32(n as i32)])
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
        }
    }

    fn push_token(
        data: &mut ChatBotUi,
        _inst: &mut Instance,
        frame: &mut CallingFrame,
        args: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        if !data.running_controller.load(Ordering::Acquire) {
            return Ok(vec![WasmValue::from_i32(-1)]);
        }

        let mem = frame
            .memory_mut(0)
            .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

        if let Some([buf_ptr, buf_size]) = args.get(0..2) {
            let buf_ptr = buf_ptr.to_i32() as usize;
            let buf_size = buf_size.to_i32() as usize;

            let r = if buf_ptr != 0 {
                let buf = mem
                    .mut_slice::<u8>(buf_ptr, buf_size)
                    .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

                data.send_output(Ok(buf))
            } else {
                data.send_output(Err(TokenError::EndOfSequence))
            };

            Ok(vec![WasmValue::from_i32(if r { 0 } else { -1 })])
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
        }
    }

    fn return_token_error(
        data: &mut ChatBotUi,
        _inst: &mut Instance,
        _frame: &mut CallingFrame,
        args: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        if let Some(error_code) = args.get(0) {
            let error_code = error_code.to_i32();
            let token_err = match error_code {
                1 => TokenError::EndOfSequence,
                2 => TokenError::ContextFull,
                3 => TokenError::PromptTooLong,
                4 => TokenError::TooLarge,
                5 => TokenError::InvalidEncoding,
                _ => TokenError::Other,
            };

            data.send_output(Err(token_err));

            Ok(vec![])
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
        }
    }

    pub fn module(data: ChatBotUi) -> wasmedge_sdk::WasmEdgeResult<ImportObject<ChatBotUi>> {
        let mut module_builder = wasmedge_sdk::ImportObjectBuilder::new("chat_ui", data)?;
        module_builder.with_func::<(i32, i32), i32>("get_input", get_input)?;
        module_builder.with_func::<(i32, i32), i32>("push_token", push_token)?;
        module_builder.with_func::<i32, ()>("return_token_error", return_token_error)?;

        Ok(module_builder.build())
    }

    fn create_wasi(
        file: &DownloadedFile,
        load_model: &LoadModelOptions,
    ) -> wasmedge_sdk::WasmEdgeResult<WasiModule> {
        let ctx_size = if load_model.n_ctx > 0 {
            Some(load_model.n_ctx.to_string())
        } else {
            None
        };

        let n_gpu_layers = match load_model.gpu_layers {
            moxin_protocol::protocol::GPULayers::Specific(n) => Some(n.to_string()),
            moxin_protocol::protocol::GPULayers::Max => None,
        };

        let batch_size = if load_model.n_batch > 0 {
            Some(load_model.n_batch.to_string())
        } else {
            None
        };

        let mut prompt_template = load_model.prompt_template.clone();
        if prompt_template.is_none() && !file.prompt_template.is_empty() {
            prompt_template = Some(file.prompt_template.clone());
        }

        let reverse_prompt = if file.reverse_prompt.is_empty() {
            None
        } else {
            Some(file.reverse_prompt.clone())
        };

        let module_alias = file.name.as_ref();

        let mut args = vec!["chat_ui.wasm", "-a", module_alias];

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

        WasiModule::create(Some(args), None, None)
    }

    pub fn nn_preload_file(file: &DownloadedFile) {
        let file_path = Path::new(&file.download_dir)
            .join(&file.model_id)
            .join(&file.name);

        let preloads = wasmedge_sdk::plugin::NNPreload::new(
            file.name.clone(),
            wasmedge_sdk::plugin::GraphEncoding::GGML,
            wasmedge_sdk::plugin::ExecutionTarget::AUTO,
            &file_path,
        );
        wasmedge_sdk::plugin::PluginManager::nn_preload(vec![preloads]);
    }

    pub fn run_wasm_by_downloaded_file(
        wasm_module: Module,
        request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
        model_running_controller: Arc<AtomicBool>,
        file: DownloadedFile,
        load_model: LoadModelOptions,
        tx: Sender<anyhow::Result<LoadModelResponse>>,
    ) {
        use wasmedge_sdk::vm::SyncInst;
        use wasmedge_sdk::AsInstance;

        let mut instances: HashMap<String, &mut (dyn SyncInst)> = HashMap::new();

        let mut wasi = create_wasi(&file, &load_model).unwrap();
        let mut chatui = module(ChatBotUi::new(
            request_rx,
            model_running_controller,
            file,
            load_model,
            tx,
        ))
        .unwrap();

        instances.insert(wasi.name().to_string(), wasi.as_mut());
        let mut wasi_nn = wasmedge_sdk::plugin::PluginManager::load_plugin_wasi_nn().unwrap();
        instances.insert(wasi_nn.name().unwrap(), &mut wasi_nn);
        instances.insert(chatui.name().unwrap(), &mut chatui);

        let store = Store::new(None, instances).unwrap();
        let mut vm = Vm::new(store);
        vm.register_module(None, wasm_module.clone()).unwrap();

        let _ = vm.run_func(None, "_start", []);

        log::debug!("wasm exit");
    }

    pub struct Model {
        pub model_tx: Sender<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
        pub model_running_controller: Arc<AtomicBool>,
        pub model_thread: JoinHandle<()>,
    }

    impl Model {
        pub fn new_by_downloaded_file(
            wasm_module: Module,
            file: DownloadedFile,
            options: LoadModelOptions,
            tx: Sender<anyhow::Result<LoadModelResponse>>,
        ) -> Self {
            let (model_tx, request_rx) = std::sync::mpsc::channel();
            let model_running_controller = Arc::new(AtomicBool::new(false));
            let model_running_controller_ = model_running_controller.clone();

            let model_thread = std::thread::spawn(move || {
                run_wasm_by_downloaded_file(
                    wasm_module,
                    request_rx,
                    model_running_controller_,
                    file,
                    options,
                    tx,
                )
            });
            Self {
                model_tx,
                model_thread,
                model_running_controller,
            }
        }

        pub fn chat(
            &self,
            data: ChatRequestData,
            tx: Sender<anyhow::Result<ChatResponse>>,
        ) -> bool {
            self.model_tx.send((data, tx)).is_ok()
        }

        pub fn stop_chat(&self) {
            self.model_running_controller
                .store(false, Ordering::Release);
        }

        pub fn stop(self) {
            let Self {
                model_tx,
                model_thread,
                ..
            } = self;
            drop(model_tx);
            let _ = model_thread.join();
        }
    }
}

#[derive(Clone, Debug)]
enum ModelManagementCommand {
    GetFeaturedModels(Sender<anyhow::Result<Vec<Model>>>),
    SearchModels(String, Sender<anyhow::Result<Vec<Model>>>),
    DownloadFile(FileID, Sender<anyhow::Result<FileDownloadResponse>>),
    PauseDownload(FileID, Sender<anyhow::Result<()>>),
    CancelDownload(FileID, Sender<anyhow::Result<()>>),
    GetCurrentDownloads(Sender<anyhow::Result<Vec<PendingDownload>>>),
    GetDownloadedFiles(Sender<anyhow::Result<Vec<DownloadedFile>>>),
}

#[derive(Clone, Debug)]
enum ModelInteractionCommand {
    LoadModel(
        FileID,
        LoadModelOptions,
        Sender<anyhow::Result<LoadModelResponse>>,
    ),
    EjectModel(Sender<anyhow::Result<()>>),
    Chat(ChatRequestData, Sender<anyhow::Result<ChatResponse>>),
    StopChatCompletion(Sender<anyhow::Result<()>>),
    // Command to start a local server to interact with chat models
    StartLocalServer(
        LocalServerConfig,
        Sender<anyhow::Result<LocalServerResponse>>,
    ),
    // Command to stop the local server
    StopLocalServer(Sender<anyhow::Result<()>>),
}

#[derive(Clone, Debug)]
enum BuiltInCommand {
    Model(ModelManagementCommand),
    Interaction(ModelInteractionCommand),
}

impl From<Command> for BuiltInCommand {
    fn from(value: Command) -> Self {
        match value {
            Command::GetFeaturedModels(tx) => {
                Self::Model(ModelManagementCommand::GetFeaturedModels(tx))
            }
            Command::SearchModels(request, tx) => {
                Self::Model(ModelManagementCommand::SearchModels(request, tx))
            }
            Command::DownloadFile(file_id, tx) => {
                Self::Model(ModelManagementCommand::DownloadFile(file_id, tx))
            }
            Command::PauseDownload(file_id, tx) => {
                Self::Model(ModelManagementCommand::PauseDownload(file_id, tx))
            }
            Command::CancelDownload(file_id, tx) => {
                Self::Model(ModelManagementCommand::CancelDownload(file_id, tx))
            }
            Command::GetCurrentDownloads(tx) => {
                Self::Model(ModelManagementCommand::GetCurrentDownloads(tx))
            }
            Command::GetDownloadedFiles(tx) => {
                Self::Model(ModelManagementCommand::GetDownloadedFiles(tx))
            }
            Command::LoadModel(file_id, options, tx) => {
                Self::Interaction(ModelInteractionCommand::LoadModel(file_id, options, tx))
            }
            Command::EjectModel(tx) => Self::Interaction(ModelInteractionCommand::EjectModel(tx)),
            Command::Chat(request, tx) => {
                Self::Interaction(ModelInteractionCommand::Chat(request, tx))
            }
            Command::StopChatCompletion(tx) => {
                Self::Interaction(ModelInteractionCommand::StopChatCompletion(tx))
            }
            Command::StartLocalServer(config, tx) => {
                Self::Interaction(ModelInteractionCommand::StartLocalServer(config, tx))
            }
            Command::StopLocalServer(tx) => {
                Self::Interaction(ModelInteractionCommand::StopLocalServer(tx))
            }
        }
    }
}

#[test]
fn test_chat() {
    use moxin_protocol::open_ai::*;

    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::GetDownloadedFiles(tx);
    bk.send(cmd).unwrap();
    let files = rx.recv().unwrap();
    assert!(files.is_ok());
    let files = files.unwrap();
    let file = files.first().unwrap();
    println!("{} {}", &file.file.id, &file.model.name);

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::LoadModel(
        file.file.id.clone(),
        LoadModelOptions {
            prompt_template: None,
            gpu_layers: moxin_protocol::protocol::GPULayers::Max,
            use_mlock: false,
            n_batch: 512,
            n_ctx: 512,
            rope_freq_scale: 0.0,
            rope_freq_base: 0.0,
            context_overflow_policy: moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
        },
        tx,
    );
    bk.send(cmd).unwrap();
    let r = rx.recv();
    assert!(r.is_ok());
    assert!(r.unwrap().is_ok());

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::Chat(
        ChatRequestData {
            messages: vec![Message {
                content: "hello".to_string(),
                role: Role::User,
                name: None,
            }],
            model: "llama-2-7b-chat.Q5_K_M".to_string(),
            frequency_penalty: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            presence_penalty: None,
            seed: None,
            stop: None,
            stream: Some(false),
            temperature: None,
            top_p: None,
            n: None,
            logit_bias: None,
        },
        tx,
    );
    bk.send(cmd).unwrap();
    if let Ok(Ok(ChatResponse::ChatFinalResponseData(data))) = rx.recv() {
        println!("{:?}", data.choices[0].message);
    }

    let (tx, rx) = std::sync::mpsc::channel();

    bk.send(Command::EjectModel(tx)).unwrap();
    rx.recv().unwrap().unwrap();
}

#[test]
fn test_chat_stop() {
    use moxin_protocol::open_ai::*;

    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::GetDownloadedFiles(tx);
    bk.send(cmd).unwrap();
    let files = rx.recv().unwrap();
    assert!(files.is_ok());
    let files = files.unwrap();
    let file = files.first().unwrap();
    println!("{} {}", &file.file.id, &file.model.name);

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::LoadModel(
        file.file.id.clone(),
        LoadModelOptions {
            prompt_template: None,
            gpu_layers: moxin_protocol::protocol::GPULayers::Max,
            use_mlock: false,
            n_batch: 512,
            n_ctx: 512,
            rope_freq_scale: 0.0,
            rope_freq_base: 0.0,
            context_overflow_policy: moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
        },
        tx,
    );
    bk.send(cmd).unwrap();
    let r = rx.recv();
    assert!(r.is_ok());
    assert!(r.unwrap().is_ok());

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::Chat(
        ChatRequestData {
            messages: vec![Message {
                content: "hello".to_string(),
                role: Role::User,
                name: None,
            }],
            model: "llama-2-7b-chat.Q5_K_M".to_string(),
            frequency_penalty: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            presence_penalty: None,
            seed: None,
            stop: None,
            stream: Some(true),
            temperature: None,
            top_p: None,
            n: None,
            logit_bias: None,
        },
        tx,
    );
    bk.send(cmd).unwrap();

    let mut i = 0;
    while let Ok(Ok(ChatResponse::ChatResponseChunk(data))) = rx.recv() {
        i += 1;
        println!(
            "{:?} {:?}",
            data.choices[0].delta, data.choices[0].finish_reason
        );
        if i == 5 {
            let (tx, rx) = std::sync::mpsc::channel();
            let cmd = Command::StopChatCompletion(tx);
            bk.send(cmd).unwrap();
            rx.recv().unwrap().unwrap();
        }
        if matches!(data.choices[0].finish_reason, Some(StopReason::Stop)) {
            break;
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();

    bk.send(Command::EjectModel(tx)).unwrap();
    rx.recv().unwrap().unwrap();
}

#[test]
fn test_download_file() {
    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::SearchModels("llama".to_string(), tx);
    bk.send(cmd).unwrap();
    let models = rx.recv().unwrap();
    assert!(models.is_ok());
    let models = models.unwrap();
    println!("{models:?}");

    let file = models[0].files[0].clone();
    println!("download {file:?}");

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::DownloadFile(file.id, tx.clone());
    bk.send(cmd).unwrap();

    let file = models[0].files[1].clone();
    println!("download {file:?}");

    let cmd = Command::DownloadFile(file.id, tx);
    bk.send(cmd).unwrap();

    println!();

    while let Ok(r) = rx.recv() {
        match r {
            Ok(FileDownloadResponse::Progress(file_id, progress)) => {
                println!("{file_id} progress: {:.2}%", progress);
            }
            Ok(FileDownloadResponse::Completed(file)) => {
                println!("Completed {file:?}");
            }
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }
}

#[test]
fn test_get_download_file() {
    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::GetDownloadedFiles(tx);
    let _ = bk.send(cmd);

    let files = rx.recv().unwrap();

    println!("{files:?}");
}

pub enum DownloadControlCommand {
    Stop,
}

pub struct BackendImpl {
    sql_conn: Arc<Mutex<rusqlite::Connection>>,
    #[allow(unused)]
    home_dir: String,
    models_dir: String,
    pub rx: Receiver<Command>,
    download_tx: crossbeam::channel::Sender<(
        store::models::Model,
        store::download_files::DownloadedFile,
        Sender<anyhow::Result<FileDownloadResponse>>,
        std::sync::mpsc::Receiver<DownloadControlCommand>,
    )>,
    model: Option<chat_ui::Model>,

    // Channels to control download threads
    download_control_channels: HashMap<FileID, std::sync::mpsc::Sender<DownloadControlCommand>>,
}

impl BackendImpl {
    /// # Argument
    ///
    /// * `home_dir` - The home directory of the application.
    /// * `models_dir` - The download path of the model.
    /// * `max_download_threads` - Maximum limit on simultaneous file downloads.
    pub fn build_command_sender(
        home_dir: String,
        models_dir: String,
        max_download_threads: usize,
    ) -> Sender<Command> {
        wasmedge_sdk::plugin::PluginManager::load(None).unwrap();

        let sql_conn = rusqlite::Connection::open(format!("{home_dir}/data.sql")).unwrap();

        // TODO Reorganize these bunch of functions, needs a little more of thought
        let _ = store::models::create_table_models(&sql_conn);
        let _ = store::download_files::create_table_download_files(&sql_conn);
        let _ = store::pending_downloads::create_table_pending_downloads(&sql_conn);
        let _ = store::pending_downloads::mark_pending_downloads_as_paused(&sql_conn);

        let sql_conn = Arc::new(Mutex::new(sql_conn));

        let (download_tx, download_rx) = crossbeam::channel::unbounded();
        let download_rx = Arc::new(download_rx);
        let (tx, rx) = std::sync::mpsc::channel();

        for _ in 0..max_download_threads.max(1) {
            let sql_conn_ = sql_conn.clone();
            let download_rx_ = download_rx.clone();

            std::thread::spawn(move || {
                store::download_file_loop(sql_conn_, download_rx_);
            });
        }

        let mut backend = Self {
            sql_conn,
            home_dir,
            models_dir,
            rx,
            download_tx,
            model: None,
            download_control_channels: HashMap::new(),
        };

        std::thread::spawn(move || {
            backend.run_loop();
        });
        tx
    }

    fn handle_command(&mut self, wasm_module: &Module, built_in_cmd: BuiltInCommand) {
        match built_in_cmd {
            BuiltInCommand::Model(file) => match file {
                ModelManagementCommand::GetFeaturedModels(tx) => {
                    let res = store::RemoteModel::get_featured_model(100, 0);
                    match res {
                        Ok(remote_model) => {
                            let sql_conn = self.sql_conn.lock().unwrap();
                            let models = RemoteModel::to_model(&remote_model, &sql_conn)
                                .map_err(|e| anyhow::anyhow!("get featured error: {e}"));

                            let _ = tx.send(models);
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("get featured models error: {e}")));
                        }
                    }
                }
                ModelManagementCommand::SearchModels(search_text, tx) => {
                    let res = store::RemoteModel::search(&search_text, 100, 0);
                    match res {
                        Ok(remote_model) => {
                            let sql_conn = self.sql_conn.lock().unwrap();
                            let models = RemoteModel::to_model(&remote_model, &sql_conn)
                                .map_err(|e| anyhow::anyhow!("search models error: {e}"));

                            let _ = tx.send(models);
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("search models error: {e}")));
                        }
                    }
                }
                ModelManagementCommand::DownloadFile(file_id, tx) => {
                    //search model from remote
                    let search_model_from_remote = || -> anyhow::Result<( crate::store::models::Model , crate::store::download_files::DownloadedFile)> {
                        let (model_id, file) = file_id
                            .split_once("#")
                            .ok_or_else(|| anyhow::anyhow!("Illegal file_id"))?;
                        let mut res = store::RemoteModel::search(&model_id, 10, 0)
                            .map_err(|e| anyhow::anyhow!("search models error: {e}"))?;
                        let remote_model = res
                            .pop()
                            .ok_or_else(|| anyhow::anyhow!("model not found"))?;

                        let remote_file = remote_model
                            .files
                            .into_iter()
                            .find(|f| f.name == file)
                            .ok_or_else(|| anyhow::anyhow!("file not found"))?;

                        let download_model = crate::store::models::Model {
                            id: Arc::new(remote_model.id),
                            name: remote_model.name,
                            summary: remote_model.summary,
                            size: remote_model.size,
                            requires: remote_model.requires,
                            architecture: remote_model.architecture,
                            released_at: remote_model.released_at,
                            prompt_template: remote_model.prompt_template.clone(),
                            reverse_prompt: remote_model.reverse_prompt.clone(),
                            author: Arc::new(crate::store::models::Author {
                                name: remote_model.author.name,
                                url: remote_model.author.url,
                                description: remote_model.author.description,
                            }),
                            like_count: remote_model.like_count,
                            download_count: remote_model.download_count,
                        };

                        let download_file = crate::store::download_files::DownloadedFile {
                            id: Arc::new(file_id.clone()),
                            model_id: model_id.to_string(),
                            name: file.to_string(),
                            size: remote_file.size,
                            quantization: remote_file.quantization,
                            prompt_template: remote_model.prompt_template,
                            reverse_prompt: remote_model.reverse_prompt,
                            downloaded: true,
                            file_size: 0,
                            download_dir: self.models_dir.clone(),
                            downloaded_at: Utc::now(),
                            tags:remote_file.tags,
                            featured: false,
                        };

                        Ok((download_model,download_file))
                    };

                    match search_model_from_remote() {
                        Ok((model, file)) => {
                            let (control_tx, control_rx) = std::sync::mpsc::channel();

                            // TODO We need to define a way to clean up the channel when the download is finished
                            self.download_control_channels
                                .insert(file_id.clone(), control_tx);

                            let _ = self.download_tx.send((model, file, tx, control_rx));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e));
                        }
                    }
                }

                ModelManagementCommand::PauseDownload(file_id, tx) => {
                    if let Some(control_tx) = self.download_control_channels.remove(&file_id) {
                        let _ = control_tx.send(DownloadControlCommand::Stop);
                    }
                    let _ = tx.send(Ok(()));
                }

                ModelManagementCommand::CancelDownload(file_id, tx) => {
                    if let Some(control_tx) = self.download_control_channels.remove(&file_id) {
                        let _ = control_tx.send(DownloadControlCommand::Stop);
                    }

                    let conn = self.sql_conn.lock().unwrap();
                    let _ = store::download_files::DownloadedFile::remove(
                        file_id.clone().into(),
                        &conn,
                    );
                    let _ = PendingDownloads::remove(file_id.clone().into(), &conn);
                    let _ = store::remove_downloaded_file(self.models_dir.clone(), file_id);

                    let _ = tx.send(Ok(()));
                }

                ModelManagementCommand::GetDownloadedFiles(tx) => {
                    let downloads = {
                        let conn = self.sql_conn.lock().unwrap();
                        store::get_all_download_file(&conn)
                            .map_err(|e| anyhow::anyhow!("get download file error: {e}"))
                    };

                    let _ = tx.send(downloads);
                }

                ModelManagementCommand::GetCurrentDownloads(tx) => {
                    let pending_downloads = {
                        let conn = self.sql_conn.lock().unwrap();
                        store::get_all_pending_downloads(&conn)
                            .map_err(|e| anyhow::anyhow!("get pending download file error: {e}"))
                    };
                    let _ = tx.send(pending_downloads);
                }
            },
            BuiltInCommand::Interaction(model_cmd) => match model_cmd {
                ModelInteractionCommand::LoadModel(file_id, options, tx) => {
                    let conn = self.sql_conn.lock().unwrap();
                    let download_file =
                        store::download_files::DownloadedFile::get_by_id(&conn, &file_id);

                    match download_file {
                        Ok(file) => {
                            chat_ui::nn_preload_file(&file);
                            let model = chat_ui::Model::new_by_downloaded_file(
                                wasm_module.clone(),
                                file,
                                options,
                                tx,
                            );
                            if let Some(old_model) = self.model.replace(model) {
                                old_model.stop();
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("Load model error: {e}")));
                        }
                    }
                }
                ModelInteractionCommand::EjectModel(tx) => {
                    if let Some(model) = self.model.take() {
                        model.stop();
                    }
                    let _ = tx.send(Ok(()));
                }
                ModelInteractionCommand::Chat(data, tx) => {
                    if let Some(model) = &self.model {
                        model.chat(data, tx);
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Model not loaded")));
                    }
                }
                ModelInteractionCommand::StopChatCompletion(tx) => {
                    self.model.as_ref().map(|model| model.stop_chat());
                    let _ = tx.send(Ok(()));
                }
                ModelInteractionCommand::StartLocalServer(_, _) => todo!(),
                ModelInteractionCommand::StopLocalServer(_) => todo!(),
            },
        }
    }

    fn run_loop(&mut self) {
        static WASM: &[u8] = include_bytes!("../chat_ui.wasm");
        let wasm_module = Module::from_bytes(None, WASM).unwrap();

        loop {
            if let Ok(cmd) = self.rx.recv() {
                self.handle_command(&wasm_module, cmd.into());
            } else {
                break;
            }
        }

        log::debug!("BackendImpl stop");
    }
}
