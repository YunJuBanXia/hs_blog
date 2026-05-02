use axum::Router;
use blog::{captcha, db::init_db, user};


#[tokio::main]
async fn main() {
    // 加载 .env 文件中的环境变量
    dotenvy::dotenv().ok();

    let pool = match init_db().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to initialize the database: {}", e);
            std::process::exit(1);
        }
    };

    let app = Router::new()
        .nest("api/", user::urls::router())
        .nest("api/", captcha::urls::router())
        // 将数据库连接池注入到应用状态中
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}
