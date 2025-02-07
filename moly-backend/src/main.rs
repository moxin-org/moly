// Standard library
use std::convert::Infallible;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

// Axum and HTTP-related
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{Request, StatusCode};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};

// Async and streaming
use futures_util::Stream;
use tokio::sync::RwLock;

// Internal crate imports
use api_errors::*;
use backend_impls::{BackendImpl, LlamaEdgeApiServerBackend};
use filesystem::{project_dirs, setup_model_downloads_folder};

// Protocol
use moly_protocol::data::{DownloadedFile, Model, PendingDownload};
use moly_protocol::open_ai::{ChatRequestData, ChatResponse};
use moly_protocol::protocol::{
    FileDownloadResponse, LoadModelRequest, LoadModelResponse, StartDownloadRequest,
};

use serde::Deserialize;

// Module declarations
mod api_errors;
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
) -> Result<Json<Vec<DownloadedFile>>, ApiErrorResponse> {
    state
        .backend
        .read()
        .await
        .get_downloaded_files()
        .map(Json)
        .map_err(internal_error)
}

/// Delete a file.
async fn delete_file(
    State(state): State<Arc<ApiState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<()>, ApiErrorResponse> {
    state
        .backend
        .read()
        .await
        .delete_file(id)
        .map(Json)
        .map_err(internal_error)
}

/// List all current downloads.
async fn list_current_downloads(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<PendingDownload>>, ApiErrorResponse> {
    state
        .backend
        .read()
        .await
        .get_current_downloads()
        .map(Json)
        .map_err(internal_error)
}

/// Start or resume downloading a model file
async fn start_download(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<StartDownloadRequest>,
) -> Result<StatusCode, ApiErrorResponse> {
    state
        .backend
        .write()
        .await
        .start_download(request.file_id)
        .await
        .map(|_| StatusCode::ACCEPTED)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, &e.to_string(), Some("file_id")))
}

/// Stream download progress via Server-Sent Events (SSE)
/// It returns 404 if the download is not in progress (regardless of it not existing, being paused, or completed).
async fn download_progress(
    State(state): State<Arc<ApiState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiErrorResponse> {
    // Fetch the corresponding progress channel
    let mut rx = state
        .backend
        .write()
        .await
        .get_download_progress_channel(id)
        .await
        .map_err(|e| {
            api_error(
                StatusCode::NOT_FOUND,
                &format!("Download not found: {}", e),
                None,
            )
        })?;

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
) -> Result<Json<()>, ApiErrorResponse> {
    state
        .backend
        .read()
        .await
        .pause_download(id)
        .map(Json)
        .map_err(internal_error)
}

/// Cancel a download.
async fn cancel_download(
    State(state): State<Arc<ApiState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<()>, ApiErrorResponse> {
    state
        .backend
        .read()
        .await
        .cancel_download(id)
        .map(Json)
        .map_err(internal_error)
}

/// Load a model.
async fn load_model(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<LoadModelRequest>,
) -> Result<Json<LoadModelResponse>, ApiErrorResponse> {
    state
        .backend
        .write()
        .await
        .load_model(request.file_id, request.options)
        .await
        .map(Json)
        .map_err(internal_error)
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

/// Search for models.
async fn search_models(
    State(state): State<Arc<ApiState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Model>>, ApiErrorResponse> {
    state
        .backend
        .write()
        .await
        .search_models(query.q)
        .map(Json)
        .map_err(internal_error)
}

/// Get featured models.
async fn get_featured_models(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<Model>>, ApiErrorResponse> {
    state
        .backend
        .write()
        .await
        .get_featured_models()
        .map(Json)
        .map_err(internal_error)
}

/// Eject a model.
async fn eject_model(State(state): State<Arc<ApiState>>) -> Result<StatusCode, ApiErrorResponse> {
    state.backend.write().await.eject_model().await;
    Ok(StatusCode::NO_CONTENT)
}

/// Chat completions endpoint
async fn chat_completions(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<ChatRequestData>,
) -> Result<Response, ApiErrorResponse> {
    let is_stream = request.stream.unwrap_or(false);
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    let result = state.backend.read().await.chat(request, tx);
    if let Err(e) = result {
        return Err(api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            e.to_string().as_str(),
            None,
        ));
    }

    if !is_stream {
        match rx.recv().await {
            Some(Ok(response)) => match response {
                ChatResponse::ChatFinalResponseData(data) => Ok(Json(data).into_response()),
                ChatResponse::ChatResponseChunk(_) => Err(api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unexpected streaming response in non-streaming mode",
                    None,
                )),
            },
            _ => Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Chat service disconnected",
                None,
            )),
        }
    } else {
        let stream = async_stream::stream! {
            while let Some(response) = rx.recv().await {
                match response {
                    Ok(ChatResponse::ChatResponseChunk(chunk)) => {
                        let event = Event::default().data(serde_json::to_string(&chunk).unwrap());
                        yield Ok::<_, std::convert::Infallible>(event);

                        if chunk.choices[0].finish_reason.is_some() {
                            yield Ok(Event::default().data("[DONE]"));
                            break;
                        }
                    }
                    Ok(ChatResponse::ChatFinalResponseData(_)) => {
                        break;
                    }
                    Err(e) => {
                        yield Ok(Event::default().data(format!("error: {}", e)));
                        break;
                    }
                }
            }
        };

        Ok(Sse::new(stream).into_response())
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let app_data_dir = project_dirs().data_dir();
    let models_dir = setup_model_downloads_folder();

    let state = Arc::new(ApiState::new(app_data_dir, models_dir).await);

    let app = Router::new()
        .nest("/files", file_routes())
        .nest("/downloads", download_routes())
        .nest("/models", model_routes())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .on_request(|request: &Request<_>, _: &_| {
                    log::debug!("--> {} {}", request.method(), request.uri());
                })
                .on_response(|response: &Response<_>, latency: Duration, _: &_| {
                    log::debug!("<-- {} ({:?})", response.status(), latency);
                }),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("🚀 server running on {}", listener.local_addr().unwrap());
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
        // TODO: We might want to provide an option to skip this /load step and do the loading automatically for the user,
        // whenever the completions endpoint is used.
        // We could have the API user porivde the model ID in the request instead of the current hardcoded "moly-chat" model.
        // (This overall depends on how we want to handle the UI loading model animations, etc.)
        .route("/load", post(load_model))
        .route("/eject", post(eject_model))
        .route("/featured", get(get_featured_models))
        .route("/search", get(search_models))
        .route("/v1/chat/completions", post(chat_completions))
    // .route("/models_dir", post(update_models_dir)) // Not sure if we will support this, or how.
}
