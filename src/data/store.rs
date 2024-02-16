use chrono::{NaiveDate, Utc};

#[derive(Debug, Clone)]
pub struct File {
    pub path: String,
    pub size: String,
    pub quantization: String,
    pub downloaded: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub description: String,
}

// We're using the HuggingFace identifier as the model ID for now
// We should consider using a different identifier in the future if more
// models sources are added.
#[derive(Debug, Clone)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: NaiveDate,
    pub files: Vec<File>,
    pub author: Author,
}

pub struct Store {
    pub models: Vec<Model>,
}

impl Store {
    pub fn new() -> Store {
        let open_hermes_files = vec![
            File {
                path: "stablelm-zephyr-3b.Q4_K_S.gguf".to_string(),
                size: "1.62 GB".to_string(),
                quantization: "Q4_K_S".to_string(),
                downloaded: true,
                tags: vec!["Small & Fast".to_string()],
            },
            File {
                path: "stablelm-zephyr-3b.Q6_K.gguf".to_string(),
                size: "2.30 GB".to_string(),
                quantization: "Q6_K".to_string(),
                downloaded: false,
                tags: vec!["Less Compressed".to_string(), "Might be slower".to_string()],
            },
        ];

        let nexus_raven_files = vec![
            File {
                path: "nexusraven-v2-13b.Q4_K_S.gguf".to_string(),
                size: "7.41 GB".to_string(),
                quantization: "Q4_K_S".to_string(),
                downloaded: false,
                tags: vec!["Small & Fast".to_string()],
            },
            File {
                path: "nexusraven-v2-13b.Q6_K.gguf".to_string(),
                size: "10.68 GB".to_string(),
                quantization: "Q6_K".to_string(),
                downloaded: true,
                tags: vec!["Less Compressed".to_string(), "Might be slower".to_string()],
            },
        ];

        let stable_lm_files = vec![
            File {
                path: "nexusraven-v2-13b.Q4_K_S.gguf".to_string(),
                size: "1.62 GB".to_string(),
                quantization: "Q4_K_S".to_string(),
                downloaded: true,
                tags: vec!["Small & Fast".to_string()],
            },
            File {
                path: "nexusraven-v2-13b.Q6_K.gguf".to_string(),
                size: "2.30 GB".to_string(),
                quantization: "Q6_K".to_string(),
                downloaded: false,
                tags: vec!["Less Compressed".to_string(), "Might be slower".to_string()],
            },
        ];

        let qwen_files = vec![
            File {
                path: "qwen1_5-7b-chat-q5_k_m.gguf".to_string(),
                size: "2.30 GB".to_string(),
                quantization: "Q5_K_M".to_string(),
                downloaded: false,
                tags: vec!["Less Compressed".to_string(), "Might be slower".to_string()],
            },
        ];

        Store {
            models: vec![
                Model {
                    id: "teknium/OpenHermes-2.5-Mistral-7B".to_string(),
                    name: "OpenHermes 2.5 Mistral 7B".to_string(),
                    summary: "OpenHermes 2.5 Mistral 7B is an advanced iteration of the OpenHermes 2 language model, enhanced by training on a significant proportion of code datasets. This additional training improved performance across several benchmarks, notably TruthfulQA, AGIEval, and the GPT4All suite, while slightly decreasing the BigBench score. Notably, the model's ability to handle code-related tasks, measured by the humaneval score...".to_string(),
                    size: "7B params".to_string(),
                    requires: "8GB+ RAM".to_string(),
                    released_at: NaiveDate::from_ymd_opt(2023, 10, 29).unwrap(),
                    architecture: "Mistral".to_string(),
                    files: open_hermes_files,
                    author: Author {
                        name: "Teknium".to_string(),
                        url: "https://github.com/teknium1".to_string(),
                        description: "Creator of numerous chart topping fine-tunes and a Co-founder of NousResearch.".to_string(),
                    },
                },
                Model {
                    id: "Nexusflow/NexusRaven-V2-13B".to_string(),
                    name: "NexusRaven-V2-13B".to_string(),
                    summary: "NexusRaven-V2 accepts a list of python functions. These python functions can do anything (e.g. sending GET/POST requests to external APIs). The two requirements include the python function signature and the appropriate docstring to generate the function call. *** Follow NexusRaven's prompting guide found on the model's Hugging Face page. ***".to_string(),
                    size: "13B params".to_string(),
                    requires: "16GB+ RAM".to_string(),
                    architecture: "LLaMa".to_string(),
                    released_at: NaiveDate::from_ymd_opt(2023, 12, 11).unwrap(),
                    files: nexus_raven_files,
                    author: Author {
                        name: "Nexusflow".to_string(),
                        url: "https://nexusflow.ai/".to_string(),
                        description: "Nexusflow is democratizing Cyber Intelligence with Generative AI, fully on top of open-source large language models (LLMs).".to_string(),
                    },
                },
                Model {
                    id: "stabilityai/stablelm-zephyr-3b".to_string(),
                    name: "StableLM Zephyr 3B".to_string(),
                    summary: "StableLM Zephyr 3B is an English-language, auto-regressive language model with 3 billion parameters, developed by Stability AI. It's an instruction-tuned model influenced by HuggingFace's Zephyr 7B training approach and is built on transformer decoder architecture. It was trained using a mix of public and synthetic datasets, including SFT and Preference Datasets from the HuggingFace Hub with Direct Preference Optimization (DPO). Its performance has been evaluated using the MT Bench and Alpaca Benchmark, achieving a score of 6.64 and a win rate of 76% respectively. For fine-tuning, it utilizes the StabilityAI's stablelm-3b-4e1t model and is available under the StabilityAI Non-Commercial Research Community License. Commercial use requires contacting Stability AI for more information. The model was trained on a Stability AI cluster with 8 nodes, each equipped with 8 A100 80GB GPUs, using internal scripts for SFT steps and HuggingFace's Alignment Handbook scripts for DPO training.".to_string(),
                    size: "3B params".to_string(),
                    requires: "8GB+ RAM".to_string(),
                    released_at: NaiveDate::from_ymd_opt(2023, 11, 21).unwrap(),
                    architecture: "StableLM".to_string(),
                    files: stable_lm_files,
                    author: Author {
                        name: "Stability AI".to_string(),
                        url: "https://stability.ai/".to_string(),
                        description: "Stability AI is developing cutting-edge open AI models for Image, Language, Audio, Video, 3D and Biology.".to_string(),
                    },
                },
                Model {
                    id: "Qwen/Qwen1.5-7B-Chat-GGUF".to_string(),
                    name: "Qwen 1.5".to_string(),
                    summary: "Qwen1.5 is the large language model series developed by Qwen Team, Alibaba Group. It is a transformer-based decoder-only language model pretrained on large-scale multilingual data covering a wide range of domains and it is aligned with human preferences.".to_string(),
                    size: "3B params".to_string(),
                    requires: "8GB+ RAM".to_string(),
                    released_at: NaiveDate::from_ymd_opt(2024, 2, 3).unwrap(),
                    architecture: "qwen2".to_string(),
                    files: qwen_files,
                    author: Author {
                        name: "Qwen Team, Alibaba Group".to_string(),
                        url: "https://huggingface.co/Qwen".to_string(),
                        description: "Qwen (abbr. for Tongyi Qianwen 通义千问) refers to the large language model family built by Alibaba Cloud".to_string(),
                    },
                },
            ],
        }
    }
}

impl Model {
    pub fn formatted_release_date(&self) -> String {
        let released_at = self.released_at.format("%b %-d, %C%y");
        let days_ago = (Utc::now().date_naive() - self.released_at).num_days();
        format!("{} ({} days ago)", released_at, days_ago)
    }
}