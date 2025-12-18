use blog_backend::serve;
#[tokio::main]
async fn main() {
    serve::serve().await;
}
