mod playground;
mod syntax;

#[tokio::main]
async fn main() {
    playground::run().await;
    syntax::r#async::tokio::demo::run().await;
}
