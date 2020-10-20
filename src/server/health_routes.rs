#[get("/start")]
pub async fn start() -> &'static str {
    "OK!"
}

#[get("/stop")]
pub async fn stop() -> &'static str {
    "OK!"
}

#[get("/health")]
pub async fn health() -> &'static str {
    "OK!"
}
