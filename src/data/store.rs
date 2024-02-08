use chrono::NaiveDate;

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

#[derive(Debug, Clone)]
pub struct Model {
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

        Store {
            models: vec![
                Model {
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
            ],
        }
    }
}