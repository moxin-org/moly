pub struct File {
    pub path: String,
    pub size: String,
    pub quantization: String,
    pub downloaded: bool,
    pub tags: Vec<String>,
}

pub struct Model {
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub files: Vec<File>,
}

pub struct Store {
    pub models: Vec<Model>,
}

impl Store {
    pub fn new() -> Store {
        let files = vec![
            File {
                path: "stablelm-zephyr-3b.Q4_K_S.gguf".to_string(),
                size: "1.62 GB".to_string(),
                quantization: "Q4_K_S".to_string(),
                downloaded: false,
                tags: vec!["Small & Fast".to_string()],
            },
            File {
                path: "stablelm-zephyr-3b.Q6_K.gguf".to_string(),
                size: "2.30 GB".to_string(),
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
                    architecture: "Mistral".to_string(),
                    files: files,
                },
            ],
        }
    }
}