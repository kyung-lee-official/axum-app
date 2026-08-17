mod syntax;

#[tokio::main]
async fn main() {
    syntax::r#async::tokio::demo::run().await;
}
