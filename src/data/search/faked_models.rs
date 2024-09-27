use moly_protocol::data::{File, Model};

use lipsum::lipsum;
use rand::distributions::{Alphanumeric, DistString};
use rand::Rng;

fn random_string(size: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), size)
}

pub fn get_faked_models(models: &Vec<Model>) -> Vec<Model> {
    let mut rng = rand::thread_rng();

    models
        .iter()
        .map(|model| {
            // filling model attributes
            let mut new_model = model.clone();
            if model.summary.is_empty() {
                new_model.summary = lipsum(30);
            }

            if model.name.is_empty() {
                // we might need a fancier word generator
                new_model.name = format!(
                    "{}-{}-{}{}-{}-{}",
                    lipsum(1),
                    rng.gen_range(0..10),
                    random_string(1).to_uppercase(),
                    rng.gen_range(0..10),
                    lipsum(1),
                    lipsum(1),
                );
            }

            if model.size.is_empty() {
                new_model.size = format!("{}B", rng.gen_range(1..10));
            };

            if model.requires.is_empty() {
                new_model.requires = match rng.gen_range(0..3) {
                    0 => "4GB+ RAM".to_string(),
                    1 => "8GB+ RAM".to_string(),
                    2 => "16GB+ RAM".to_string(),
                    _ => "32GB+ RAM".to_string(),
                };
            }

            if model.architecture.is_empty() {
                new_model.architecture = match rng.gen_range(0..3) {
                    0 => "Mistral".to_string(),
                    1 => "StableLM".to_string(),
                    2 => "LlaMa".to_string(),
                    _ => "qwen2".to_string(),
                };
            }

            if model.like_count == 0 {
                new_model.like_count = rng.gen_range(1..1000);
            };

            if model.download_count == 0 {
                new_model.download_count = rng.gen_range(0..10000);
            };

            // filling files
            let new_files: Vec<File> = model
                .files
                .iter()
                .map(|file| {
                    let mut new_file = file.clone();

                    if new_file.quantization.is_empty() {
                        new_file.quantization = format!(
                            "Q{}_{}_{}",
                            rng.gen_range(0..10),
                            random_string(1).to_uppercase(),
                            random_string(1).to_uppercase()
                        );
                    }

                    if file.name.is_empty() {
                        // we might need a fancier word generator
                        new_file.name = format!(
                            "{}-{}-{}-{}-{}.{}.gguf",
                            lipsum(1),
                            rng.gen_range(0..10),
                            random_string(5),
                            lipsum(1),
                            new_file.quantization,
                            rng.gen_range(0..10),
                        );
                    }

                    if file.size.is_empty() {
                        new_file.size = rng.gen_range(100000000..999999999).to_string();
                    };

                    new_file.featured = rng.gen_bool(0.15);

                    new_file
                })
                .collect();

            new_model.files = new_files;
            new_model
        })
        .collect()
}
