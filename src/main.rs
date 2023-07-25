use quilkin::filters::StaticFilter;
use tracing::Level;
use tracing_subscriber::EnvFilter;

mod delay;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // register custom filters
    quilkin::filters::FilterRegistry::register(vec![delay::Delay::factory()].into_iter());

    // run quilkin CLI almost as normal
    // see: https://github.com/googleforgames/quilkin/blob/92b4920b9024bb7ef30dc8711d923370dd2cdf44/src/main.rs#L22
    let mut cli = <quilkin::Cli as clap::Parser>::parse();
    cli.quiet = true;
    match cli.drive().await {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            tracing::error!(%error, error_debug=?error, "fatal error");
            std::process::exit(-1)
        }
    }
}
