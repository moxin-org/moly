//! Miscellaneous platform (operating system) utilities that don't fit elsewhere
//! or are limited to a specific platform functionality.

/// From bytes, create a blob url that is accessible inside the provided closure.
///
/// This URL will be revoked after the closure is executed.
///
/// Web only.
#[cfg(target_arch = "wasm32")]
pub(crate) fn create_scoped_blob_url(
    content: &[u8],
    content_type: Option<&str>,
    f: impl FnOnce(&str),
) {
    let uint8_array = js_sys::Uint8Array::new_with_length(content.len() as u32);
    uint8_array.copy_from(content);

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&uint8_array);

    let blob_options = web_sys::BlobPropertyBag::new();
    if let Some(content_type) = content_type {
        blob_options.set_type(content_type);
    }

    let blob =
        web_sys::Blob::new_with_blob_sequence_and_options(&blob_parts, &blob_options).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    let url = url.as_str();

    f(url);

    web_sys::Url::revoke_object_url(url).unwrap();
}

/// Cause the underlying platform to start downloading a file.
///
/// Web only.
#[cfg(target_arch = "wasm32")]
pub(crate) fn trigger_download(url: &str, filename: &str) {
    use web_sys::wasm_bindgen::JsCast;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let a = document.create_element("a").unwrap();
    let a = a.dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    a.set_attribute("href", url).unwrap();
    a.set_attribute("download", filename).unwrap();
    a.click();
    a.remove();
}

/// Prompts the user to save a file with the given content.
///
/// Native platforms only.
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub(crate) fn trigger_save_as(content: &[u8], filename: Option<&str>) {
    let filename = filename.unwrap_or("file");
    let dialog = rfd::FileDialog::new().set_file_name(filename);

    let Some(path) = dialog.save_file() else {
        return;
    };

    if let Err(e) = std::fs::write(&path, content) {
        makepad_widgets::error!(
            "Failed to save file '{}' to '{}': {}",
            filename,
            path.display(),
            e
        );
    }
}
