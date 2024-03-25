use std::sync::mpsc::{Receiver, Sender};

use chrono::Utc;
use moxin_protocol::{
    data::{Author, CompatibilityGuess, DownloadedFile, File, FileID, Model},
    open_ai::{ChatRequestData, ChatResponse},
    protocol::{
        Command, FileDownloadResponse, LoadModelOptions, LoadModelResponse, LocalServerConfig,
        LocalServerResponse,
    },
};
use wasmedge_sdk::Module;

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
        sync::mpsc::{Receiver, Sender},
        thread::JoinHandle,
    };

    use moxin_protocol::{
        data::FileID,
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

    #[derive(Debug)]
    pub struct ChatBotUi {
        pub current_req: std::io::Cursor<Vec<u8>>,
        pub request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
        request_id: uuid::Uuid,
        chat_completion_message: Option<Vec<u8>>,
        pub token_tx: Option<Sender<anyhow::Result<ChatResponse>>>,
        pub load_model_state: Option<(
            FileID,
            LoadModelOptions,
            Sender<anyhow::Result<LoadModelResponse>>,
        )>,
    }

    impl ChatBotUi {
        pub fn new(
            request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
            load_module_req: (
                FileID,
                LoadModelOptions,
                Sender<anyhow::Result<LoadModelResponse>>,
            ),
        ) -> Self {
            Self {
                request_rx,
                request_id: uuid::Uuid::new_v4(),
                token_tx: None,
                current_req: std::io::Cursor::new(vec![]),
                load_model_state: Some(load_module_req),
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
                if let Some((file_id, _, tx)) = data.load_model_state.take() {
                    let file_id = file_id.clone();
                    let model_id = file_id.clone();
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
        module_alias: &str,
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

        let prompt_template = load_model.prompt_template.clone();

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

        WasiModule::create(Some(args), None, None)
    }

    pub fn list_models(models_dir: &str) -> Vec<(String, String)> {
        let mut r = vec![];

        if let Ok(read_dir) = std::fs::read_dir(models_dir) {
            for dir_entry in read_dir {
                if let Ok(dir_entry) = dir_entry {
                    let path = dir_entry.path();
                    if path.is_file() && "gguf" == path.extension().unwrap_or_default() {
                        let file_stem = path.file_stem().unwrap_or(path.as_os_str());

                        r.push((
                            format!("{}", path.display()),
                            format!("{}", file_stem.to_string_lossy()),
                        ));
                    }
                }
            }
        }
        r
    }

    pub fn nn_preload(models_dir: &str) {
        let models = list_models(models_dir);

        let preloads = models
            .into_iter()
            .map(|(path, file_stem)| {
                wasmedge_sdk::plugin::NNPreload::new(
                    file_stem,
                    wasmedge_sdk::plugin::GraphEncoding::GGML,
                    wasmedge_sdk::plugin::ExecutionTarget::AUTO,
                    path,
                )
            })
            .collect();

        wasmedge_sdk::plugin::PluginManager::nn_preload(preloads);
    }

    pub fn run_wasm(
        wasm_module: Module,
        request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
        model_id: String,
        load_model_commond: (
            FileID,
            LoadModelOptions,
            Sender<anyhow::Result<LoadModelResponse>>,
        ),
    ) {
        use wasmedge_sdk::vm::SyncInst;
        use wasmedge_sdk::AsInstance;

        let mut instances: HashMap<String, &mut (dyn SyncInst)> = HashMap::new();

        let mut wasi = create_wasi(&model_id, &load_model_commond.1).unwrap();
        let mut chatui = module(ChatBotUi::new(request_rx, load_model_commond)).unwrap();

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
        pub model_thread: JoinHandle<()>,
    }

    impl Model {
        pub fn new(
            wasm_module: Module,
            file_id: FileID,
            options: LoadModelOptions,
            tx: Sender<anyhow::Result<LoadModelResponse>>,
        ) -> Self {
            let (model_tx, request_rx) = std::sync::mpsc::channel();

            let model_thread = std::thread::spawn(|| {
                run_wasm(
                    wasm_module,
                    request_rx,
                    file_id.clone(),
                    (file_id, options, tx),
                )
            });
            Self {
                model_tx,
                model_thread,
            }
        }

        pub fn chat(
            &self,
            data: ChatRequestData,
            tx: Sender<anyhow::Result<ChatResponse>>,
        ) -> bool {
            self.model_tx.send((data, tx)).is_ok()
        }

        pub fn stop(self) {
            let Self {
                model_tx,
                model_thread,
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
    let bk = BackendImpl::build_command_sender(format!("{home}/ai/models"));

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
        file.model.name.clone(),
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
fn test_download_file() {
    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::build_command_sender(format!("{home}/ai/models"));

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::SearchModels("llama".to_string(), tx);
    bk.send(cmd).unwrap();
    let models = rx.recv().unwrap();
    assert!(models.is_ok());
    let models = models.unwrap();
    println!("{models:?}");

    let file = models[0].files[0].clone();
    println!("{file:?}");

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::DownloadFile(file.id, tx);
    bk.send(cmd).unwrap();

    while let Ok(r) = rx.recv() {
        match r {
            Ok(FileDownloadResponse::Progress(_, progress)) => {
                println!("progress: {:.2}%", progress);
            }
            Ok(FileDownloadResponse::Completed(file)) => {
                println!("Completed {file:?}");
                break;
            }
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }
}

pub struct BackendImpl {
    models_dir: String,
    pub rx: Receiver<Command>,
    model: Option<chat_ui::Model>,
}

impl BackendImpl {
    /// # Argument
    ///
    /// * `models_dir` - The download path of the model.
    pub fn build_command_sender(models_dir: String) -> Sender<Command> {
        wasmedge_sdk::plugin::PluginManager::load(None).unwrap();
        chat_ui::nn_preload(&models_dir);

        let (tx, rx) = std::sync::mpsc::channel();
        let mut backend = Self {
            models_dir,
            rx,
            model: None,
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
                    // TODO: Featured Models have not been set up yet, so return an empty list here.
                    let _ = tx.send(Ok(vec![]));
                }
                ModelManagementCommand::SearchModels(search_text, tx) => {
                    let res = super::model_manager::search(&search_text, 100, 0);
                    let _ = tx.send(res.map_err(|e| anyhow::anyhow!("search models error: {e}")));
                }
                ModelManagementCommand::DownloadFile(file_id, tx) => {
                    let mut send_progress = |progress| {
                        let _ = tx.send(Ok(FileDownloadResponse::Progress(
                            file_id.clone(),
                            progress as f32,
                        )));
                    };

                    if let Some((id, file)) = file_id.split_once("#") {
                        let r = super::model_manager::download_file_from_huggingface(
                            id,
                            file,
                            &self.models_dir,
                            0.5,
                            &mut send_progress,
                        );

                        match r {
                            Ok(_) => {
                                let local_path = format!("{}/{}", self.models_dir, file);

                                let _ =
                                    tx.send(Ok(FileDownloadResponse::Completed(DownloadedFile {
                                        file: File {
                                            id: file_id.clone(),
                                            name: file.to_string(),
                                            size: String::new(),
                                            quantization: String::new(),
                                            downloaded: true,
                                            downloaded_path: Some(local_path),
                                            tags: Vec::new(),
                                            featured: false,
                                        },
                                        model: Model::default(),
                                        downloaded_at: Utc::now().date_naive(),
                                        compatibility_guess: CompatibilityGuess::PossiblySupported,
                                        information: String::new(),
                                    })));
                            }
                            Err(e) => tx
                                .send(Err(anyhow::anyhow!("Download failed: {e}")))
                                .unwrap(),
                        }
                    };
                }

                ModelManagementCommand::GetDownloadedFiles(tx) => {
                    let files = chat_ui::list_models(&self.models_dir)
                        .into_iter()
                        .enumerate()
                        .map(|(id, (path, file_stem))| DownloadedFile {
                            file: File {
                                id: id.to_string(),
                                name: path,
                                size: "3.08 GB".to_string(),
                                quantization: String::new(),
                                downloaded: true,
                                downloaded_path: None,
                                tags: vec![],
                                featured: true,
                            },
                            model: Model {
                                id: id.to_string(),
                                name: file_stem.clone(),
                                summary: format!("summary of {file_stem}"),
                                size: format!("size of {file_stem}"),
                                requires: String::new(),
                                architecture: String::new(),
                                released_at: Utc::now(),
                                files: vec![],
                                author: Author {
                                    name: format!("author of {file_stem}"),
                                    url: String::new(),
                                    description: String::new(),
                                },
                                like_count: 0,
                                download_count: 0,
                            },
                            downloaded_at: chrono::Utc::now().date_naive(),
                            compatibility_guess:
                                moxin_protocol::data::CompatibilityGuess::PossiblySupported,
                            information: String::new(),
                        })
                        .collect();
                    let _ = tx.send(Ok(files));
                }
            },
            BuiltInCommand::Interaction(model_cmd) => match model_cmd {
                ModelInteractionCommand::LoadModel(file_id, options, tx) => {
                    chat_ui::nn_preload(&self.models_dir);
                    let model = chat_ui::Model::new(wasm_module.clone(), file_id, options, tx);
                    self.model = Some(model);
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
                ModelInteractionCommand::StopChatCompletion(_) => todo!(),
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
