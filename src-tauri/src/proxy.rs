use axum::{
    body::Body,
    extract::{Json, Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Router,
};
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, instrument, warn};

use crate::models::{ProxyServerConfig, ProxyStatus, RequestContext, RoutingDecision};
use crate::routing::{RoutingEngine, RoutingReason};

/// Shared state for the proxy server
#[derive(Clone)]
pub struct ProxyState {
    pub config: Arc<RwLock<ProxyServerConfig>>,
    pub routing_engine: Arc<RoutingEngine>,
    pub http_client: reqwest::Client,
    pub request_count: Arc<AtomicU64>,
    pub start_time: Arc<RwLock<Option<Instant>>>,
}

impl ProxyState {
    pub fn new(routing_engine: Arc<RoutingEngine>, config: ProxyServerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            routing_engine,
            http_client: reqwest::Client::new(),
            request_count: Arc::new(AtomicU64::new(0)),
            start_time: Arc::new(RwLock::new(None)),
        }
    }
}

/// Health check response
#[derive(Debug, serde::Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
}

/// The proxy server
pub struct ProxyServer {
    state: ProxyState,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    bind_addr: String,
}

impl ProxyServer {
    pub fn new(routing_engine: Arc<RoutingEngine>, config: ProxyServerConfig) -> Self {
        let bind_addr = config.bind_addr.clone();
        Self {
            state: ProxyState::new(routing_engine, config),
            shutdown_tx: None,
            bind_addr,
        }
    }

    /// Start the proxy server
    pub async fn start(&mut self) -> anyhow::Result<()> {
        let config = self.state.config.read().await.clone();
        let addr: SocketAddr = config.bind_addr.parse()
            .map_err(|e| anyhow::anyhow!("Invalid bind address: {}", e))?;

        let app = Self::build_router(self.state.clone());

        let listener = tokio::net::TcpListener::bind(addr).await
            .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

        info!("Proxy server listening on http://{}", addr);

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Mark start time
        *self.state.start_time.write().await = Some(Instant::now());

        let state = self.state.clone();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                    info!("Proxy server shutting down gracefully");
                })
                .await
                .expect("Server failed");
        });

        Ok(())
    }

    /// Build the Axum router
    fn build_router(state: ProxyState) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .route("/health", get(health_handler))
            .route("/healthz", get(health_handler))
            .route("/v1/models", get(list_models_handler))
            .route("/v1/chat/completions", post(chat_completions_handler))
            .route("/v1/completions", post(completions_handler))
            .route("/v1/embeddings", post(embeddings_handler))
            .route("/v1/images/generations", post(images_handler))
            .route("/*path", any(proxy_handler))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .layer(TraceLayer::new_for_http())
            .layer(cors)
            .with_state(state)
    }

    /// Stop the server
    pub fn stop(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Get current status
    pub async fn get_status(&self) -> ProxyStatus {
        let start_time = *self.state.start_time.read().await;
        let uptime_seconds = start_time
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);

        ProxyStatus {
            running: self.shutdown_tx.is_some(),
            bind_addr: self.bind_addr.clone(),
            request_count: self.state.request_count.load(Ordering::Relaxed),
            uptime_seconds,
        }
    }
}

/// Authentication middleware
async fn auth_middleware(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let config = state.config.read().await;

    if let Some(provided_key) = auth_header {
        if provided_key == config.api_key {
            drop(config);
            return Ok(next.run(request).await);
        }
    }

    warn!("Unauthorized request: invalid or missing API key");
    Err(StatusCode::UNAUTHORIZED)
}

/// Health check handler
async fn health_handler(State(state): State<ProxyState>) -> impl IntoResponse {
    let stats = state.routing_engine.get_stats().await;
    let start_time = *state.start_time.read().await;
    let uptime_seconds = start_time
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);

    Json(HealthResponse {
        status: if stats.available_accounts > 0 {
            "ok"
        } else {
            "degraded"
        }
        .to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
    })
}

/// List models handler
async fn list_models_handler() -> impl IntoResponse {
    let models = serde_json::json!({
        "object": "list",
        "data": [
            { "id": "gpt-4", "object": "model", "owned_by": "openai" },
            { "id": "gpt-4-turbo", "object": "model", "owned_by": "openai" },
            { "id": "gpt-4o", "object": "model", "owned_by": "openai" },
            { "id": "gpt-4o-mini", "object": "model", "owned_by": "openai" },
            { "id": "gpt-3.5-turbo", "object": "model", "owned_by": "openai" },
            { "id": "text-embedding-3-small", "object": "model", "owned_by": "openai" },
            { "id": "text-embedding-3-large", "object": "model", "owned_by": "openai" },
            { "id": "dall-e-3", "object": "model", "owned_by": "openai" },
        ]
    });

    Json(models)
}

/// Chat completions handler
async fn chat_completions_handler(
    State(state): State<ProxyState>,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, StatusCode> {
    handle_openai_request(state, body, "/v1/chat/completions").await
}

/// Completions handler
async fn completions_handler(
    State(state): State<ProxyState>,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, StatusCode> {
    handle_openai_request(state, body, "/v1/completions").await
}

/// Embeddings handler
async fn embeddings_handler(
    State(state): State<ProxyState>,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, StatusCode> {
    handle_openai_request(state, body, "/v1/embeddings").await
}

/// Images handler
async fn images_handler(
    State(state): State<ProxyState>,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, StatusCode> {
    handle_openai_request(state, body, "/v1/images/generations").await
}

/// Generic proxy handler
async fn proxy_handler(
    State(state): State<ProxyState>,
    request: Request<Body>,
) -> Result<impl IntoResponse, StatusCode> {
    let path = request.uri().path().to_string();
    
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let body: Value = if body_bytes.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?
    };

    handle_openai_request(state, body, &path).await
}

/// Core request handling logic
#[instrument(skip(state, body), fields(model = %body.get("model").and_then(|v| v.as_str()).unwrap_or("unknown")))]
async fn handle_openai_request(
    state: ProxyState,
    body: Value,
    path: &str,
) -> Result<impl IntoResponse, StatusCode> {
    state.request_count.fetch_add(1, Ordering::Relaxed);

    let model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4")
        .to_string();

    let session_id = extract_session_id(&body);

    let ctx = RequestContext::new(model.clone())
        .with_session(session_id.clone().unwrap_or_default());

    let decision = match state.routing_engine.resolve_account(&ctx).await {
        Ok(d) => d,
        Err(e) => {
            warn!("Routing failed: {}", e);
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    debug!(
        "Routing {} request to account {} ({})",
        path, decision.account_label, decision.account_id
    );

    let config = state.config.read().await;
    let url = format!("{}{}", config.openai_base_url, path);
    drop(config);

    let upstream_req = state
        .http_client
        .request(reqwest::Method::POST, &url)
        .header("Authorization", format!("Bearer {}", decision.api_key))
        .header("Content-Type", "application/json");

    let upstream_req = if let Some(org_id) = &decision.org_id {
        upstream_req.header("OpenAI-Organization", org_id)
    } else {
        upstream_req
    };

    let is_streaming = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);

    let upstream_resp = upstream_req
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            error!("Upstream request failed: {}", e);
            state.routing_engine.report_error(decision.account_id, true);
            StatusCode::BAD_GATEWAY
        })?;

    let status = upstream_resp.status();

    if !status.is_success() {
        let error_body = upstream_resp
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        warn!("Upstream error {}: {}", status, error_body);
        state.routing_engine.report_error(decision.account_id, status.as_u16() >= 500);

        return Ok(Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(error_body))
            .unwrap());
    }

    state.routing_engine.report_success(decision.account_id);

    if is_streaming {
        let stream = upstream_resp.bytes_stream().map(move |result| {
            result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        });

        let body = Body::from_stream(stream);

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(body)
            .unwrap());
    }

    let response_body = upstream_resp
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(response_body))
        .unwrap())
}

/// Extract session ID from request body
fn extract_session_id(body: &Value) -> Option<String> {
    if let Some(messages) = body.get("messages").and_then(|v| v.as_array()) {
        if let Some(first_msg) = messages.first() {
            if let Some(content) = first_msg.get("content").and_then(|v| v.as_str()) {
                use sha2::{Digest, Sha256};
                let hash = Sha256::digest(content.as_bytes());
                return Some(format!("{:x}", &hash[..8]));
            }
        }
    }
    None
}

use axum::Json;

/// Global proxy server instance (managed by Tauri state)
static PROXY_SERVER: tokio::sync::RwLock<Option<ProxyServer>> = tokio::sync::RwLock::const_new(None);

/// Tauri command: Start proxy server
#[tauri::command]
pub async fn start_proxy_server(
    routing_engine: tauri::State<'_, Arc<RoutingEngine>>,
    config: ProxyServerConfig,
) -> Result<(), String> {
    let mut server = PROXY_SERVER.write().await;
    
    if server.is_some() {
        return Err("Proxy server already running".to_string());
    }

    let mut new_server = ProxyServer::new(routing_engine.inner().clone(), config);
    new_server.start().await.map_err(|e| e.to_string())?;
    
    *server = Some(new_server);
    Ok(())
}

/// Tauri command: Stop proxy server
#[tauri::command]
pub async fn stop_proxy_server() -> Result<(), String> {
    let mut server = PROXY_SERVER.write().await;
    
    if let Some(s) = server.take() {
        s.stop();
        Ok(())
    } else {
        Err("Proxy server not running".to_string())
    }
}

/// Tauri command: Get proxy status
#[tauri::command]
pub async fn get_proxy_status() -> Result<ProxyStatus, String> {
    let server = PROXY_SERVER.read().await;
    
    if let Some(s) = server.as_ref() {
        Ok(s.get_status().await)
    } else {
        Ok(ProxyStatus {
            running: false,
            bind_addr: "127.0.0.1:8080".to_string(),
            request_count: 0,
            uptime_seconds: 0,
        })
    }
}
