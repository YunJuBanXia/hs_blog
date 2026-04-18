use axum::{Extension, Router};
use blog::{db::init_db, user};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("api/", user::urls::router())
        .layer(Extension(init_db().await.expect("Failed to initialize the database")));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}
