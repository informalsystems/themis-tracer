use {anyhow::Result, env_logger, log, std::env, tracer::cmd};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let log_level = env::var("RUST_LOG").unwrap_or("warn".to_string());
    log::info!("log level set to {}", log_level);
    cmd::opt::run()
}
