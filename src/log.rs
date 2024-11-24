use anyhow::{Context as _, Result};

#[derive(Debug)]
pub struct LogFileArgs {
    pub rolling: tracing_appender::rolling::Rotation,
    pub directory: String,
    pub name_prefix: String,
}

impl Default for LogFileArgs {
    fn default() -> Self {
        Self { 
            rolling: tracing_appender::rolling::Rotation::DAILY, 
            directory: Default::default(), 
            name_prefix: Default::default() 
        }
    }
}

#[derive(Default)]
pub struct LogGuard {
    _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}


#[macro_export]
macro_rules! init_log {
    () => {
        $crate::log::init_log_with(env!("CARGO_CRATE_NAME"), env!("CARGO_PKG_NAME"), &[], None)
    };

    ($extra_mods:expr) => {
        $crate::log::init_log_with(env!("CARGO_CRATE_NAME"), env!("CARGO_PKG_NAME"), $extra_mods, None)
    };

    ($extra_mods:expr, $file_args:expr) => {
        $crate::log::init_log_with(env!("CARGO_CRATE_NAME"), env!("CARGO_PKG_NAME"), $extra_mods, $file_args)
    }
}

pub fn init_log_with(crate_name: &str, packet_name: &str, extra_mods: &[&str], file_args: Option<&LogFileArgs>) -> Result<LogGuard> {


    // println!("CARGO_PKG_NAME: {}", env!("CARGO_PKG_NAME"));
    // println!("CARGO_CRATE_NAME: {}", env!("CARGO_CRATE_NAME"));


    // https://time-rs.github.io/book/api/format-description.html

    // // let fmts = time::macros::format_description!("[hour]:[minute]:[second].[subsecond digits:3]");
    // let fmts = time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]");

    // // let offset = time::UtcOffset::current_local_offset().expect("should get local offset!");
    // // let timer = tracing_subscriber::fmt::time::OffsetTime::new(offset, fmts);

    // let timer = tracing_subscriber::fmt::time::LocalTime::new(fmts); 

    // let timer = tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%dT%H:%M:%S%.3f".into());
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new("%T.%3f".into());
    
    // let filter = if cfg!(debug_assertions) {
    //     if let Ok(v) = std::env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV) {
    //         v.into()
    //     } else {
    //         // let crate_name = env!("CARGO_CRATE_NAME");
    //         // let packet_name = env!("CARGO_PKG_NAME");
    //         let underline_crate_name = crate_name.replace("-", "_");
    //         let underline_packet_name = packet_name.replace("-", "_");
    //         let mut expr = format!("{underline_crate_name}=debug");
    //         if packet_name != crate_name {
    //             expr.push_str(",");
    //             expr.push_str(&format!("{underline_packet_name}=debug"));
    //         }

    //         for extra in extra_mods {
    //             expr.push_str(",");
    //             expr.push_str(&format!("{extra}=debug"));
    //         }
            
    //         expr.into()
    //     }
    // } else {
    //     tracing_subscriber::EnvFilter::builder()
    //     .with_default_directive(tracing::level_filters::LevelFilter::DEBUG.into())
    //     .from_env_lossy()
    // };

    let filter: tracing_subscriber::EnvFilter = if let Ok(v) = std::env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV) {
        v.into()
    } else {
        // let crate_name = env!("CARGO_CRATE_NAME");
        // let packet_name = env!("CARGO_PKG_NAME");
        let underline_crate_name = crate_name.replace("-", "_");
        let underline_packet_name = packet_name.replace("-", "_");
        let mut expr = format!("{underline_crate_name}=debug");
        if packet_name != crate_name {
            expr.push_str(",");
            expr.push_str(&format!("{underline_packet_name}=debug"));
        }

        for extra in extra_mods {
            expr.push_str(",");
            expr.push_str(&format!("{extra}=debug"));
        }
        
        expr.into()
    };

    let ansi = if file_args.is_some() {
        false
    } else {
        if cfg!(debug_assertions) {
            true
        } else {
            false
        }
    };

    let builder =     tracing_subscriber::fmt()
        .with_max_level(tracing::metadata::LevelFilter::DEBUG)
        .with_env_filter(filter)
        // .with_env_filter("rtun=debug,rserver=debug")
        // .with_writer(w)
        .with_timer(timer)
        .with_target(false)
        .with_ansi(ansi);
        // .with_ansi(!file_args.is_some());

    if let Some(file_args) = file_args {
        let log_path = std::path::Path::new(&file_args.directory);

        if !log_path.exists() {
            std::fs::create_dir_all(log_path)
                .with_context(||format!("failed to make log dir [{}]", file_args.directory))?;
        } else {
            if !log_path.is_dir() {
                anyhow::bail!("log path must be directory [{}]", file_args.directory)
            }
        }

        let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
            .filename_prefix(&file_args.name_prefix)
            .filename_suffix("log")
            .rotation(file_args.rolling.clone())
            .max_log_files(30)
            .build(&file_args.directory)
            .with_context(||"build log file appender faile")?;

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        builder
            .with_writer(non_blocking)
            .init();
        Ok(LogGuard {
            _guard: Some(guard),
        })
    } else {
        builder.init();
        Ok(LogGuard {
            _guard: None,
        })
    }

}
