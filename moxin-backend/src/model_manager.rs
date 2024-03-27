use std::fs::File;
use std::io::{self, Read, Seek, Write};

use moxin_protocol::data::Model;

fn get_file_content_length(client: &reqwest::blocking::Client, url: &str) -> reqwest::Result<u64> {
    let response = client.head(url).send()?;

    let content_length = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0);

    Ok(content_length)
}

fn download_file(
    client: &reqwest::blocking::Client,
    content_length: u64,
    url: &str,
    local_path: &str,
    step: f64,
    report_fn: &mut dyn FnMut(f64),
) -> io::Result<f64> {
    println!("download_file_to {local_path}");
    let mut file = File::options()
        .write(true)
        .create(true)
        .open(local_path)
        .unwrap();
    let file_length = file.metadata()?.len();

    if file_length < content_length {
        println!(
            "Resuming download from byte {}:{}",
            file_length, content_length
        );
        file.seek(io::SeekFrom::End(0))?;

        let range = format!("bytes={}-", file_length);
        let mut resp = client
            .get(url)
            .header("Range", range)
            .send()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut buffer = vec![0; (content_length as usize) / 100];
        let mut downloaded: u64 = file_length;
        let mut last_progress = 0.0;
        loop {
            let len = match resp.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(len) => len,
                Err(e) => return Err(e),
            };
            file.write_all(&buffer[..len])?;
            downloaded += len as u64;

            let progress = (downloaded as f64 / content_length as f64) * 100.0;
            if progress > last_progress + step {
                last_progress = progress;
                report_fn(progress)
            }
        }
        Ok((downloaded as f64 / content_length as f64) * 100.0)
    } else {
        Ok(100.0)
    }
}

pub fn download_file_from_huggingface(
    id: &str,
    file: &str,
    model_dir: &str,
    step: f64,
    report_fn: &mut dyn FnMut(f64),
) -> io::Result<f64> {
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}?download=true",
        id, file
    );

    let client = reqwest::blocking::Client::new();
    let content_length = get_file_content_length(&client, &url)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let local_path = format!("{model_dir}/{file}");

    download_file(&client, content_length, &url, &local_path, step, report_fn)
}

#[test]
fn test_download_file_from_huggingface() {
    download_file_from_huggingface(
        "TheBloke/Llama-2-7B-Chat-GGUF",
        "llama-2-7b-chat.Q3_K_M.gguf",
        "/home/csh/ai/models",
        0.5,
        &mut |progress| {
            println!("Download progress: {:.2}%", progress);
        },
    )
    .unwrap();
}

fn fill_data(model: &mut Model) {
    let model_id = model.id.clone();
    for file in &mut model.files {
        file.id = format!("{model_id}#{}", file.name);
    }
}

pub fn search(search_text: &str, limit: usize, offset: usize) -> reqwest::Result<Vec<Model>> {
    let url = format!("https://code.flows.network/webhook/DsbnEK45sK3NUzFUyZ9C/models?status=published&trace_status=tracing&order=most_likes&offset={offset}&limit={limit}&search={search_text}");
    println!("get {url}");
    let response = reqwest::blocking::get(&url)?;
    let mut models: Vec<Model> = response.json()?;
    for model in &mut models {
        fill_data(model);
    }
    Ok(models)
}

#[test]
fn test_search() {
    let models = search("llama", 100, 0).unwrap();
    println!("{:?}", models);
}
