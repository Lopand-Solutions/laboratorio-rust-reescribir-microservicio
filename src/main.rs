#[tokio::main]
async fn main() {
    let puerto = std::env::var("PUERTO").unwrap_or_else(|_| "3002".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{puerto}"))
        .await
        .unwrap();
    println!("rust escuchando en {puerto}");
    axum::serve(listener, convertidor::app())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await
        .unwrap();
}
