#![allow(dead_code)]
use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// ── State ───────────────────────────────────────────────────
struct AppState {
    start_time: Instant,
    stats: Mutex<Stats>,
}

struct Stats {
    total_compiles: u64,
    total_runs: u64,
    total_parses: u64,
    total_optimizations: u64,
    total_errors: u64,
}

// ── Types ────────────────────────────────────────────────────
#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_jobs: u64 }

// Compile
#[derive(Deserialize)]
#[allow(dead_code)]
struct CompileRequest {
    source: Option<String>,
    language: Option<String>,
    target: Option<String>,
    optimize: Option<bool>,
}
#[derive(Serialize)]
struct CompileResponse {
    job_id: String,
    status: String,
    language: String,
    target: String,
    bytecode_size_bytes: u64,
    symbol_count: u32,
    warnings: Vec<String>,
    errors: Vec<String>,
    elapsed_us: u128,
}

// Run
#[derive(Deserialize)]
#[allow(dead_code)]
struct RunRequest {
    bytecode: Option<String>,
    args: Option<Vec<String>>,
    timeout_ms: Option<u64>,
}
#[derive(Serialize)]
struct RunResponse {
    job_id: String,
    status: String,
    exit_code: i32,
    stdout: String,
    stderr: String,
    elapsed_ms: u64,
    memory_kb: u64,
}

// Parse
#[derive(Deserialize)]
#[allow(dead_code)]
struct ParseRequest {
    source: Option<String>,
    language: Option<String>,
}
#[derive(Serialize)]
struct ParseResponse {
    job_id: String,
    status: String,
    language: String,
    token_count: u32,
    ast_nodes: u32,
    parse_errors: Vec<String>,
    elapsed_us: u128,
}

// Optimize
#[derive(Deserialize)]
#[allow(dead_code)]
struct OptimizeRequest {
    bytecode: Option<String>,
    level: Option<u8>,
    passes: Option<Vec<String>>,
}
#[derive(Serialize)]
struct OptimizeResponse {
    job_id: String,
    status: String,
    level: u8,
    passes_applied: Vec<String>,
    size_before_bytes: u64,
    size_after_bytes: u64,
    reduction_pct: f64,
    elapsed_us: u128,
}

// Stats
#[derive(Serialize)]
struct StatsResponse {
    total_compiles: u64,
    total_runs: u64,
    total_parses: u64,
    total_optimizations: u64,
    total_errors: u64,
}

// ── Main ─────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "compiler_engine=info".into()))
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_compiles: 0,
            total_runs: 0,
            total_parses: 0,
            total_optimizations: 0,
            total_errors: 0,
        }),
    });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/compiler/compile", post(compile))
        .route("/api/v1/compiler/run", post(run))
        .route("/api/v1/compiler/parse", post(parse))
        .route("/api/v1/compiler/optimize", post(optimize))
        .route("/api/v1/compiler/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("COMPILER_ADDR").unwrap_or_else(|_| "0.0.0.0:8129".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Compiler Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ─────────────────────────────────────────────────
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(),
        total_jobs: st.total_compiles + st.total_runs + st.total_parses + st.total_optimizations,
    })
}

async fn compile(State(s): State<Arc<AppState>>, Json(req): Json<CompileRequest>) -> Json<CompileResponse> {
    let t = Instant::now();
    let lang = req.language.unwrap_or_else(|| "alice-dsl".into());
    let target = req.target.unwrap_or_else(|| "wasm32".into());
    let source_len = req.source.as_deref().unwrap_or("").len() as u64;

    // Estimate bytecode size and symbol count from source
    let bytecode_size = estimate_bytecode_size(source_len, &target);
    let symbol_count = estimate_symbol_count(source_len);

    // Warn on large inputs
    let warnings = if source_len > 65536 {
        vec!["Source exceeds 64KB; consider splitting into modules".into()]
    } else {
        vec![]
    };

    s.stats.lock().unwrap().total_compiles += 1;

    Json(CompileResponse {
        job_id: uuid::Uuid::new_v4().to_string(),
        status: "success".into(),
        language: lang,
        target,
        bytecode_size_bytes: bytecode_size,
        symbol_count,
        warnings,
        errors: vec![],
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn run(State(s): State<Arc<AppState>>, Json(req): Json<RunRequest>) -> Json<RunResponse> {
    let t = Instant::now();
    let timeout = req.timeout_ms.unwrap_or(5000);
    let args = req.args.unwrap_or_default();

    // Simulate execution result
    let stdout = format!("alice-vm: executed with {} arg(s), timeout={}ms", args.len(), timeout);

    s.stats.lock().unwrap().total_runs += 1;

    Json(RunResponse {
        job_id: uuid::Uuid::new_v4().to_string(),
        status: "completed".into(),
        exit_code: 0,
        stdout,
        stderr: String::new(),
        elapsed_ms: t.elapsed().as_millis() as u64,
        memory_kb: 4096,
    })
}

async fn parse(State(s): State<Arc<AppState>>, Json(req): Json<ParseRequest>) -> Json<ParseResponse> {
    let t = Instant::now();
    let lang = req.language.unwrap_or_else(|| "alice-dsl".into());
    let source_len = req.source.as_deref().unwrap_or("").len() as u32;

    // Estimate token/AST counts from source length
    let token_count = (source_len / 4).max(1);
    let ast_nodes = (source_len / 10).max(1);

    s.stats.lock().unwrap().total_parses += 1;

    Json(ParseResponse {
        job_id: uuid::Uuid::new_v4().to_string(),
        status: "success".into(),
        language: lang,
        token_count,
        ast_nodes,
        parse_errors: vec![],
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn optimize(State(s): State<Arc<AppState>>, Json(req): Json<OptimizeRequest>) -> Json<OptimizeResponse> {
    let t = Instant::now();
    let level = req.level.unwrap_or(2).min(3);
    let passes = req.passes.unwrap_or_else(|| default_passes(level));
    let size_before: u64 = req.bytecode.as_deref().map(|b| b.len() as u64).unwrap_or(16384);

    // Size reduction per optimization level
    let reduction_pct = match level {
        0 => 0.0_f64,
        1 => 12.5,
        2 => 24.0,
        3 => 35.0,
        _ => 24.0,
    };
    let size_after = (size_before as f64 * (1.0 - reduction_pct / 100.0)) as u64;

    s.stats.lock().unwrap().total_optimizations += 1;

    Json(OptimizeResponse {
        job_id: uuid::Uuid::new_v4().to_string(),
        status: "success".into(),
        level,
        passes_applied: passes,
        size_before_bytes: size_before,
        size_after_bytes: size_after,
        reduction_pct,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse {
        total_compiles: st.total_compiles,
        total_runs: st.total_runs,
        total_parses: st.total_parses,
        total_optimizations: st.total_optimizations,
        total_errors: st.total_errors,
    })
}

// ── Helpers ──────────────────────────────────────────────────
fn estimate_bytecode_size(source_bytes: u64, target: &str) -> u64 {
    let ratio = match target {
        "wasm32" | "wasm64" => 1.8,
        "x86_64" | "amd64"  => 2.4,
        "aarch64" | "arm64" => 2.2,
        "llvm-ir"           => 3.0,
        _                   => 2.0,
    };
    ((source_bytes as f64 * ratio) as u64).max(64)
}

fn estimate_symbol_count(source_bytes: u64) -> u32 {
    // Rough heuristic: one symbol per 80 bytes of source
    ((source_bytes / 80) as u32).max(1)
}

fn default_passes(level: u8) -> Vec<String> {
    let mut passes = vec!["dead-code-elimination".into(), "constant-folding".into()];
    if level >= 2 {
        passes.push("inline-expansion".into());
        passes.push("loop-unrolling".into());
    }
    if level >= 3 {
        passes.push("vectorization".into());
        passes.push("register-allocation".into());
        passes.push("instruction-scheduling".into());
    }
    passes
}
