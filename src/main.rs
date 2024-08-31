// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
#[cfg(feature = "load_config")]
mod config;
#[cfg(feature = "load_config")]
use config::*;
#[cfg(feature = "load_config")]
use figment::providers::Format;
mod main_inner;
use main_inner::*;


fn main() -> std::process::ExitCode
{
    const DEBUG: bool = true; // debug mode?
    let config: Config; // config, settings
    let tokio_rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("Creating tokio runtime failed."); // async runtime


    if DEBUG == true
    // setup logging
    {
        setup_logging::setup_logging(log::Level::Debug, None, "./log/%Y-%m-%dT%H_%M.log");
    }
    else
    {
        setup_logging::setup_logging(log::Level::Info, None, "./log/%Y-%m-%d.log");
    }

    std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
        // override panic behaviour
        log::error!("{}", panic_info); // log panic source and reason
        log::error!("{}", std::backtrace::Backtrace::capture()); // log backtrace
    }));

    #[cfg(feature = "load_config")]
    match figment::Figment::new() // load configuration
        //.merge(figment::providers::Json::file("./config/config.json")) // load "./config/config.json" file
        //.merge(figment::providers::Yaml::file("./config/config.yaml")) // load "./config/config.yaml" file
        .merge(figment::providers::Toml::file("./config/.env")) // load "./.env" file
        .merge(figment::providers::Env::raw()) // load environment variables
        .extract()
    {
        Ok(c) =>
        // loaded config successfully
        {
            config = c;
            log::debug!("Loaded {config:?}.");
        }
        Err(e) =>
        // loading config failed
        {
            log::error!("Loading config failed with: {e}");
            match e.kind
            {
                figment::error::Kind::MissingField(_) =>
                // setting unset, offer to create default config file
                {
                    Config::offer_default_file_creation("./config/.env", FileFormat::Toml);
                }
                _ =>
                    // something else went wrong, don't offer anything
                    {}
            }
            return std::process::ExitCode::FAILURE;
        }
    }


    match std::panic::catch_unwind(|| tokio_rt.block_on(main_inner(config))) // execute main_inner, catch panic
    {
        Ok(_) =>
        {
            return std::process::ExitCode::SUCCESS; // no panic, program executed successfully
        }
        Err(_) =>
        {
            return std::process::ExitCode::FAILURE; // panic, program failed
        }
    };
}