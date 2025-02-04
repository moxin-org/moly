use axum::extract::Query;
use axum::response::sse::{Event, Sse};
use axum::routing::post;
use axum::{extract::State, routing::delete};
use backend_impls::{BackendImpl, LlamaEdgeApiServerBackend};
use filesystem::{project_dirs, setup_model_downloads_folder};
use futures_util::Stream;
use moly_protocol::data::{DownloadedFile, Model, PendingDownload};
use moly_protocol::protocol::{
    FileDownloadResponse, LoadModelRequest, LoadModelResponse, StartDownloadRequest,
};
use serde::Deserialize;
use std::convert::Infallible;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{extract::Path as AxumPath, http::StatusCode, routing::get, Json, Router};

mod backend_impls;
mod filesystem;
mod store;

struct ApiState {
    // This RwLock is just a placeholder for now (and probably not a great one, as it turns out a lot of GET requests end up doing writes as well).
    // This is a placeholder because we want to make the server less stateful in general,
    // and in any case we can more granularly introduce mutexes to substates as needed.
    backend: Arc<RwLock<LlamaEdgeApiServerBackend>>,
}

impl ApiState {
    pub async fn new(app_data_dir: impl AsRef<Path>, models_dir: impl AsRef<Path>) -> Self {
        let backend = BackendImpl::build(app_data_dir, models_dir, 10).await;

        Self {
            backend: Arc::new(RwLock::new(backend)),
        }
    }
}

/// List all downloaded files.
async fn list_downloaded_files(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<DownloadedFile>>, (StatusCode, String)> {
    state
        .backend
        .read()
        .await
        .get_downloaded_files()
        .map(Json)
        .map_err(|e| internal_error(e))
}

/// Delete a file.
async fn delete_file(
    State(state): State<Arc<ApiState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<()>, (StatusCode, String)> {
    state
        .backend
        .read()
        .await
        .delete_file(id)
        .map(Json)
        .map_err(|e| internal_error(e))
}

/// List all current downloads.
async fn list_current_downloads(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<PendingDownload>>, (StatusCode, String)> {
    state
        .backend
        .read()
        .await
        .get_current_downloads()
        .map(Json)
        .map_err(|e| internal_error(e))
}

/// Start or resume downloading a model file
async fn start_download(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<StartDownloadRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .backend
        .write()
        .await
        .start_download(request.file_id)
        .await
        .map(|_| StatusCode::ACCEPTED)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

/// Stream download progress via Server-Sent Events (SSE)
/// It returns 404 if the download is not in progress (regardless of it not existing, being paused, or completed).
async fn download_progress(
    State(state): State<Arc<ApiState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    // Fetch the corresponding progress channel
    let mut rx = state
        .backend
        .write()
        .await
        .get_download_progress_channel(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Download not found: {}", e)))?;

    // Stream the progress updates
    let stream = async_stream::stream! {
        while let Some(response) = rx.recv().await {
            match response {
                Ok(FileDownloadResponse::Progress(_, progress)) => {
                    yield Ok(Event::default().event("progress").data(progress.to_string()));
                }
                Ok(FileDownloadResponse::Completed(_)) => {
                    yield Ok(Event::default().event("complete").data("100"));
                    break;
                }
                Err(_) => {
                    yield Ok(Event::default().event("error").data("Download failed"));
                    break;
                }
            }
        }
    };

    Ok(Sse::new(stream))
}

/// Pause a download.
async fn pause_download(
    State(state): State<Arc<ApiState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<()>, (StatusCode, String)> {
    state
        .backend
        .read()
        .await
        .pause_download(id)
        .map(Json)
        .map_err(|e| internal_error(e))
}

/// Cancel a download.
async fn cancel_download(
    State(state): State<Arc<ApiState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<()>, (StatusCode, String)> {
    state
        .backend
        .read()
        .await
        .cancel_download(id)
        .map(Json)
        .map_err(|e| internal_error(e))
}

/// Load a model.
async fn load_model(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<LoadModelRequest>,
) -> Result<Json<LoadModelResponse>, (StatusCode, String)> {
    state
        .backend
        .write()
        .await
        .load_model(request.file_id, request.options)
        .await
        .map(Json)
        .map_err(|e| internal_error(e))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

/// Search for models.
async fn search_models(
    State(state): State<Arc<ApiState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Model>>, (StatusCode, String)> {
    state
        .backend
        .write()
        .await
        .search_models(query.q)
        .map(Json)
        .map_err(|e| internal_error(e))
}

/// Get featured models.
async fn get_featured_models(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<Model>>, (StatusCode, String)> {
    state
        .backend
        .write()
        .await
        .get_featured_models()
        .map(Json)
        .map_err(|e| internal_error(e))
}

/// Eject a model.
async fn eject_model(
    State(state): State<Arc<ApiState>>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.backend.write().await.eject_model().await;

    Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() {
    let app_data_dir = project_dirs().data_dir();
    let models_dir = setup_model_downloads_folder();

    let state = Arc::new(ApiState::new(app_data_dir, models_dir).await);

    let app = Router::new()
        .nest("/files", file_routes())
        .nest("/downloads", download_routes())
        .nest("/models", model_routes())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// File management routes.
///
/// Note that file IDs match the IDs in model routes. From that perspective files and models are the same.
/// Models are conceptually a collection of files, but there is a lot of exisitng code (and UI langauge) that treats models as a single entity.
/// For example, loading a model basically takes a file ID and loads the corresponding file.
fn file_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/", get(list_downloaded_files))
        .route("/{id}", delete(delete_file))
}

/// Download management routes.
fn download_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/", get(list_current_downloads))
        .route("/", post(start_download))
        .route("/{id}/progress", get(download_progress))
        .route("/{id}", post(pause_download))
        .route("/{id}", delete(cancel_download))
}

/// Model management routes.
fn model_routes() -> Router<Arc<ApiState>> {
    Router::new()
        // WIP. When we introduce the completions endpoint (a proxy to the LLamaEdge API server), we might want to provide an option to
        // skip this /load step and do the loading automatically for the user, by having the user porivde the model ID in the request instead
        // of the current hardcoded "moly-chat" model. (this overall depends on how we want to handle the UI loading model animations, etc.)
        .route("/load", post(load_model))
        .route("/eject", post(eject_model))
        .route("/featured", get(get_featured_models))
        .route("/search", get(search_models))
    // .route("/models_dir", post(update_models_dir)) // Not sure if we will support this, or how.
}

/// Utility function for mapping errors into a `500 Internal Server Error`
/// response.
fn internal_error(err: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
