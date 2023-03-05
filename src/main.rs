use api_aga_in::run;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    run().await
}
