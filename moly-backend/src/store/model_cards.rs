use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;

pub static REPO_NAME: &'static str = "model-cards";

pub fn sync_model_cards_repo<P: AsRef<Path>>(app_data_dir: P) -> anyhow::Result<ModelCardManager> {
    let repo_url: &'static str =
        option_env!("MODEL_CARDS_REPO").unwrap_or("https://git.gitmono.org/project/moxin/model-cards.git");
    let repo_dirs = app_data_dir.as_ref().join(REPO_NAME);

    // Sync the repo by `libra`
    let origin_dir = std::env::current_dir()?;
    let mut r = Ok(());
    if !repo_dirs.exists() {
        log::info!("Cloning model-cards repo, url:{:?}, path: {:?}", repo_url, repo_dirs);
        libra::exec(vec!["clone", repo_url, repo_dirs.to_str().unwrap()])?;
    } else {
        log::info!("Pulling model-cards repo, path: {:?}", repo_dirs);
        // CAUTION: set_current_dir will affect of the whole process
        std::env::set_current_dir(&repo_dirs)?;
        for _ in 0..2 {
            r = libra::exec(vec!["pull", "origin", "main"]);
            if r.is_ok() {
                break;
            }
        }
    }
    std::env::set_current_dir(&origin_dir)?;

    {
        // TODO test code, remove it
        add_model_card(
            app_data_dir.as_ref(),
            Path::new("/home/bean/.local/share/moly/model_downloads/second-state/Qwen2.5-0.5B-Instruct-GGUF/Qwen2.5-0.5B-Instruct-Q2_K.gguf")
        ).expect("Failed to add model card");
    }

    if let Err(e) = r {
        log::error!("Failed to pull: {:?}", e);
        log::error!("please remove the repo({:?}) and try again", &repo_dirs);
    }

    let index_url = format!("{}/releases/download/index_release/index.json", repo_url);

    let index_list = if let Ok(remote_index) =
        reqwest::blocking::get(index_url).and_then(|r| r.json::<Vec<ModelIndex>>())
    {
        remote_index
    } else {
        log::warn!("Failed to get remote index, load local index");
        let index_list = std::fs::read_to_string(repo_dirs.join("index.json"))?;
        let index_list: Vec<ModelIndex> = serde_json::from_str(&index_list)?;
        index_list
    };

    let mut indexs = HashMap::with_capacity(index_list.len());
    for index in index_list {
        indexs.insert(index.id.clone(), index);
    }

    let embedding_index =
        if let Ok(embedding_index) = std::fs::read_to_string(repo_dirs.join("embedding.json")) {
            let embedding_index: EmbeddingIndex = serde_json::from_str(&embedding_index)?;
            if !embedding_index.check_file_exist(app_data_dir.as_ref()) {
                let app_data_dir_path = app_data_dir.as_ref().to_path_buf();
                let r = std::thread::spawn(move || {
                    if let Ok(_) = embedding_index.download(&app_data_dir_path) {
                        log::debug!("Downloaded embedding model ok");
                        Some(embedding_index)
                    } else {
                        log::warn!("Failed to download embedding model");
                        None
                    }
                });
                EmbeddingState::Pending(r)
            } else {
                EmbeddingState::Finish(Some(embedding_index))
            }
        } else {
            EmbeddingState::Finish(None)
        };

    Ok(ModelCardManager {
        app_data_dir: app_data_dir.as_ref().to_path_buf(),
        embedding_index,
        indexs,
        caches: HashMap::new(),
    })
}

// TODO replace with libra::utils::lfs::xxx
/// SHA256
// `ring` crate is much faster than `sha2` crate ( > 10 times)
fn calc_lfs_file_hash<P>(path: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 65536];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let file_hash = hex::encode(hash.finish().as_ref());
    Ok(file_hash)
}

pub fn add_model_card<P: AsRef<Path>>(app_data_dir: P, model_path: P) -> anyhow::Result<()> {
    let peer_id = &vault::get_peerid();
    let base_name =  model_path.as_ref().file_stem().unwrap().to_str().unwrap();
    let id = format!("{}/{}", peer_id, base_name); // TODO vs files-name
    let repo_dirs = app_data_dir.as_ref().join(REPO_NAME);
    let model_card_path = repo_dirs
        .join(peer_id)
        .join(format!("{}.json", base_name));

    let mut model_card = if let Ok(model_card_str) = std::fs::read_to_string(model_card_path.clone()) {
        serde_json::from_str(&model_card_str)?
    } else {
        ModelCard {
            id: id.clone(), // TODO
            name: base_name.to_string(),
            released_at: Utc::now(),
            files: vec![],
            context_size: 0, // TODO
            author: Author {
                name: peer_id.to_string(),
                url: "".to_string(),
                description: "".to_string(),
            },
            like_count: 0,
            download_count: 0,
            metrics: None,
            ..Default::default()
        }
    };

    let is_model_exist = model_card.files.iter().any(|f| f.name == base_name);
    if !is_model_exist {
        let sha256 = calc_lfs_file_hash(model_path.as_ref())?;
        let size = std::fs::metadata(model_path.as_ref())?.len();
        model_card.files.push(RemoteFile {
            name: base_name.to_string(), // TODO same with id?
            size: size.to_string(),
            quantization: "".to_string(), // TODO
            tags: vec![],
            sha256: Some(sha256.clone()),
            download: DownloadUrls {
                default: format!("p2p://{}/sha256/{}", peer_id, sha256),
            },
        });

        if let Some(parent) = model_card_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // save model card
        let model_card_str = serde_json::to_string_pretty(&model_card)?;
        let mut file = File::create(model_card_path.clone())?;
        file.write_all(model_card_str.as_bytes())?;

        log::debug!("Model card added: {:?}", id);
    } else {
        log::warn!("Model card already exists: {:?}", id);
    }

    let index_path = repo_dirs.join("index.json");
    let mut index_list = if let Ok(index_str) = std::fs::read_to_string(&index_path) {
        serde_json::from_str::<Vec<ModelIndex>>(&index_str)?
    } else {
        vec![]
    };

    let has_index = index_list.iter().any(|index| index.id == id);
    if !has_index {
        index_list.push(ModelIndex {
            id: id.clone(),
            name: model_card.name.clone(),
            architecture: model_card.architecture.clone(),
            summary: model_card.summary.clone(),
            model_type: "instruct".to_string(), // TODO
            featured: true, // TODO
            like_count: 0,
            download_count: 0,
        });

        // save index
        let index_str = serde_json::to_string_pretty(&index_list)?;
        let mut file = File::create(index_path.clone())?;
        file.write_all(index_str.as_bytes())?;

        log::debug!("Model card added to 'index.json': {:?}", id);
    } else {
        log::info!("Model card already exists in 'index.json': {:?}", id);
    }

    // commit changes
    let origin_dir = std::env::current_dir()?;
    // CAUTION: set_current_dir will affect of the whole process
    std::env::set_current_dir(&repo_dirs)?;

    libra::exec(vec!["add", model_card_path.to_str().unwrap()])?;
    libra::exec(vec!["add", index_path.to_str().unwrap()])?;
    libra::exec(vec!["commit", "-m", &format!("Add model card: {}", base_name)])?;
    libra::exec(vec!["push"])?;

    std::env::set_current_dir(&origin_dir)?;
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelIndex {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub architecture: String,
    #[serde(default)]
    pub model_type: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub like_count: u32,
    #[serde(default)]
    pub download_count: u32,
}

impl ModelIndex {
    pub fn load_model_card(&self, app_data_dir: &Path) -> anyhow::Result<ModelCard> {
        let (org_name, model_name) = self
            .id
            .split_once("/")
            .ok_or(anyhow::anyhow!("Invalid model id: {}", self.id))?;

        let sub_name = if !self.name.is_empty() {
            self.name.as_str()
        } else {
            model_name
        };

        let model_card_path = app_data_dir
            .join(REPO_NAME)
            .join(org_name)
            .join(format!("{}.json", sub_name));
        let model_card = std::fs::read_to_string(model_card_path)?;
        let mut model_card: ModelCard = serde_json::from_str(&model_card)?;
        model_card.like_count = self.like_count;
        model_card.download_count = self.download_count;

        Ok(model_card)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub description: String,
}

impl Into<moly_protocol::data::Author> for Author {
    fn into(self) -> moly_protocol::data::Author {
        moly_protocol::data::Author {
            name: self.name,
            url: self.url,
            description: self.description,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ModelCard {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub requires: String,
    #[serde(default)]
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    #[serde(default)]
    pub files: Vec<RemoteFile>,
    pub prompt_template: String,
    pub reverse_prompt: String,
    pub context_size: u64,
    pub author: Author,
    #[serde(default)]
    pub like_count: u32,
    #[serde(default)]
    pub download_count: u32,
    #[serde(default)]
    pub metrics: Option<HashMap<String, f32>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RemoteFile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub quantization: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    pub download: DownloadUrls,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DownloadUrls {
    #[serde(default)]
    pub default: String,
}

impl ModelCard {
    pub fn to_model(
        remote_models: &[Self],
        conn: &rusqlite::Connection,
    ) -> rusqlite::Result<Vec<moly_protocol::data::Model>> {
        let model_ids = remote_models
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>();
        let files = super::download_files::DownloadedFile::get_by_models(conn, &model_ids)?;

        fn to_file(
            model_id: &str,
            remote_files: &[RemoteFile],
            save_files: &HashMap<Arc<String>, super::download_files::DownloadedFile>,
        ) -> rusqlite::Result<Vec<moly_protocol::data::File>> {
            let mut files = vec![];
            for remote_f in remote_files {
                let file_id = format!("{}#{}", model_id, remote_f.name);
                let downloaded_path = save_files.get(&file_id).map(|file| {
                    let file_path = Path::new(&file.download_dir)
                        .join(&file.model_id)
                        .join(&file.name);
                    file_path
                        .to_str()
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                });

                let file = moly_protocol::data::File {
                    id: file_id,
                    name: remote_f.name.clone(),
                    size: remote_f.size.clone(),
                    quantization: remote_f.quantization.clone(),
                    downloaded: downloaded_path.is_some(),
                    downloaded_path,
                    tags: remote_f.tags.clone(),
                    featured: false,
                };

                files.push(file);
            }

            Ok(files)
        }

        let mut models = Vec::with_capacity(remote_models.len());

        for remote_m in remote_models {
            let model = moly_protocol::data::Model {
                id: remote_m.id.clone(),
                name: remote_m.name.clone(),
                summary: remote_m.summary.clone(),
                size: remote_m.size.clone(),
                requires: remote_m.requires.clone(),
                architecture: remote_m.architecture.clone(),
                released_at: remote_m.released_at.clone(),
                files: to_file(&remote_m.id, &remote_m.files, &files)?,
                author: moly_protocol::data::Author {
                    name: remote_m.author.name.clone(),
                    url: remote_m.author.url.clone(),
                    description: remote_m.author.description.clone(),
                },
                like_count: remote_m.like_count.clone(),
                download_count: remote_m.download_count.clone(),
                metrics: remote_m.metrics.clone().unwrap_or_default(),
            };

            models.push(model);
        }

        Ok(models)
    }
}

pub struct ModelCardManager {
    app_data_dir: PathBuf,
    embedding_index: EmbeddingState,
    indexs: HashMap<String, ModelIndex>,
    caches: HashMap<String, ModelCard>,
}

pub enum EmbeddingState {
    Pending(std::thread::JoinHandle<Option<EmbeddingIndex>>),
    Finish(Option<EmbeddingIndex>),
}

impl ModelCardManager {
    pub fn empty(app_data_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            indexs: HashMap::new(),
            caches: HashMap::new(),
            embedding_index: EmbeddingState::Finish(None),
        }
    }

    pub fn load_model_card(&mut self, index: &ModelIndex) -> anyhow::Result<ModelCard> {
        let r = self
            .caches
            .entry(index.id.clone())
            .or_insert(index.load_model_card(&self.app_data_dir)?);
        Ok(r.clone())
    }

    pub fn get_index_by_id(&self, id: &str) -> Option<&ModelIndex> {
        self.indexs.get(id)
    }

    pub fn search(
        &self,
        search_text: &str,
        limit: usize,
        offset: usize,
    ) -> reqwest::Result<Vec<ModelIndex>> {
        let search_text = search_text.trim().to_ascii_lowercase();
        Ok(self
            .indexs
            .values()
            .filter(|index| {
                (index.model_type == "instruct" || index.model_type == "chat")
                    && (index.name.to_ascii_lowercase().contains(&search_text)
                        || index
                            .architecture
                            .to_ascii_lowercase()
                            .contains(&search_text)
                        || index.id.to_ascii_lowercase().contains(&search_text)
                        || index.summary.to_ascii_lowercase().contains(&search_text))
            })
            .map(Clone::clone)
            .skip(offset)
            .take(limit)
            .collect::<Vec<ModelIndex>>())
    }

    pub fn get_featured_model(
        &self,
        limit: usize,
        offset: usize,
    ) -> reqwest::Result<Vec<ModelIndex>> {
        Ok(self
            .indexs
            .values()
            .filter(|index| {
                (index.model_type == "instruct" || index.model_type == "chat") && index.featured
            })
            .map(Clone::clone)
            .skip(offset)
            .take(limit)
            .collect::<Vec<ModelIndex>>())
    }

    pub fn embedding_model(&mut self) -> Option<(PathBuf, u64)> {
        match &self.embedding_index {
            EmbeddingState::Pending(res) => {
                if res.is_finished() {
                    ()
                } else {
                    return None;
                }
            }
            EmbeddingState::Finish(None) => return None,
            EmbeddingState::Finish(Some(index)) => {
                return Some((index.file_path(&self.app_data_dir), index.ctx))
            }
        };

        let state = std::mem::replace(&mut self.embedding_index, EmbeddingState::Finish(None));
        let res = match state {
            EmbeddingState::Pending(res) => res.join().unwrap(),
            EmbeddingState::Finish(_) => unreachable!(),
        };

        match res {
            Some(index) => {
                let r = (index.file_path(&self.app_data_dir), index.ctx);
                self.embedding_index = EmbeddingState::Finish(Some(index));
                Some(r)
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingIndex {
    id: String,
    file: String,
    ctx: u64,
    download: String,
}

impl EmbeddingIndex {
    pub fn file_path(&self, app_data_dir: &Path) -> PathBuf {
        app_data_dir.join("embedding").join(&self.file)
    }

    pub fn check_file_exist(&self, app_data_dir: &Path) -> bool {
        let file_path = self.file_path(app_data_dir);
        file_path.exists()
    }

    pub fn download(&self, app_data_dir: &Path) -> anyhow::Result<()> {
        let file_path = self.file_path(app_data_dir);

        let _ = std::fs::create_dir_all(file_path.parent().unwrap());

        let client = reqwest::blocking::Client::new();
        let mut resp = client.get(&self.download).send()?;
        let mut file = std::fs::File::create(file_path)?;
        std::io::copy(&mut resp, &mut file)?;
        Ok(())
    }
}
