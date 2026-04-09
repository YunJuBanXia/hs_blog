use axum::{Router};
use blog::user;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("api/", user::urls::router());

    let listener = tokio::net::TcpListener::bind("0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}
