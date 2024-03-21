use mullvad_daemon::{
    logging,
    management_interface::{ManagementInterfaceEventBroadcaster, ManagementInterfaceServer},
    rpc_uniqueness_check,
    runtime::new_runtime_builder,
    version, Daemon, DaemonCommandChannel, DaemonCommandSender,
};
use std::{path::PathBuf, thread, time::Duration};
use talpid_types::ErrorExt;

mod cli;
#[cfg(target_os = "linux")]
mod early_boot_firewall;
mod exception_logging;
#[cfg(target_os = "macos")]
mod macos_launch_daemon;
#[cfg(windows)]
mod system_service;

const DAEMON_LOG_FILENAME: &str = "daemon.log";
#[cfg(target_os = "linux")]
const EARLY_BOOT_LOG_FILENAME: &str = "early-boot-fw.log";

fn main() {
    let exit_code = match run() {
        Ok(_) => 0,
        Err(error) => {
            if logging::is_enabled() {
                log::error!("{error}");
            } else {
                eprintln!("{error}")
            }

            1
        }
    };

    log::debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

fn run() -> Result<(), String> {
    let config = cli::get_config();

    let runtime = new_runtime_builder()
        .build()
        .map_err(|e| e.display_chain().to_string())?;

    match config.command {
        cli::Command::Daemon => {
            // uniqueness check must happen before logging initializaton,
            // as initializing logs will rotate any existing log file.
            runtime.block_on(assert_unique())?;
            let log_dir = init_daemon_logging(config)?;
            log::trace!("Using configuration: {:?}", config);

            runtime.block_on(run_standalone(log_dir))
        }

        #[cfg(target_os = "linux")]
        cli::Command::InitializeEarlyBootFirewall => {
            init_early_boot_logging(config);

            runtime
                .block_on(crate::early_boot_firewall::initialize_firewall())
                .map_err(|err| format!("{err}"))
        }

        #[cfg(target_os = "windows")]
        cli::Command::RunAsService => {
            init_logger(config, None)?;
            runtime.block_on(assert_unique())?;
            system_service::run()
        }

        #[cfg(target_os = "windows")]
        cli::Command::RegisterService => {
            init_logger(config, None)?;
            system_service::install_service()
                .inspect(|_| println!("Installed the service."))
                .map_err(|e| e.display_chain())
        }

        #[cfg(target_os = "macos")]
        cli::Command::LaunchDaemonStatus => {
            if version::is_dev_version() {
                eprintln!("Note: This command may not work on non-notarized builds.");
            }

            std::process::exit(macos_launch_daemon::get_status() as i32);
        }
    }
}

/// Check that there's not another daemon currently running.
async fn assert_unique() -> Result<(), &'static str> {
    if rpc_uniqueness_check::is_another_instance_running().await {
        return Err("Another instance of the daemon is already running");
    }
    Ok(())
}

/// Initialize logging to stderr and to file (if configured).
fn init_daemon_logging(config: &cli::Config) -> Result<Option<PathBuf>, String> {
    let log_dir = get_log_dir(config)?;
    let log_path = |filename| log_dir.as_ref().map(|dir| dir.join(filename));

    init_logger(config, log_path(DAEMON_LOG_FILENAME))?;

    if let Some(ref log_dir) = log_dir {
        log::info!("Logging to {}", log_dir.display());
    }
    Ok(log_dir)
}

/// Initialize logging to stder and to the [`EARLY_BOOT_LOG_FILENAME`]
#[cfg(target_os = "linux")]
fn init_early_boot_logging(config: &cli::Config) {
    // If it's possible to log to the filesystem - attempt to do so, but failing that mustn't stop
    // the daemon from starting here.
    if let Ok(Some(log_dir)) = get_log_dir(config) {
        if init_logger(config, Some(log_dir.join(EARLY_BOOT_LOG_FILENAME))).is_ok() {
            return;
        }
    }

    let _ = init_logger(config, None);
}

/// Initialize logging to stderr and to file (if provided).
fn init_logger(config: &cli::Config, log_file: Option<PathBuf>) -> Result<(), String> {
    logging::init_logger(
        config.log_level,
        log_file.as_ref(),
        config.log_stdout_timestamps,
    )
    .map_err(|e| e.display_chain_with_msg("Unable to initialize logger"))?;
    log_panics::init();
    exception_logging::enable();
    version::log_version();
    Ok(())
}

fn get_log_dir(config: &cli::Config) -> Result<Option<PathBuf>, String> {
    if config.log_to_file {
        Ok(Some(mullvad_paths::log_dir().map_err(|e| {
            e.display_chain_with_msg("Unable to get log directory")
        })?))
    } else {
        Ok(None)
    }
}

async fn run_standalone(log_dir: Option<PathBuf>) -> Result<(), String> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    if let Err(err) = tokio::fs::remove_file(mullvad_paths::get_rpc_socket_path()).await {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::error!("Failed to remove old RPC socket: {}", err);
        }
    }

    if !running_as_admin() {
        log::warn!("Running daemon as a non-administrator user, clients might refuse to connect");
    }

    let daemon = create_daemon(log_dir).await?;

    let shutdown_handle = daemon.shutdown_handle();
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    mullvad_daemon::shutdown::set_shutdown_signal_handler(move || {
        shutdown_handle.shutdown(!mullvad_daemon::shutdown::is_shutdown_user_initiated())
    })
    .map_err(|e| e.display_chain())?;

    #[cfg(any(windows, target_os = "android"))]
    mullvad_daemon::shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown(true))
        .map_err(|e| e.display_chain())?;

    daemon.run().await.map_err(|e| e.display_chain())?;

    log::info!("Mullvad daemon is quitting");
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

async fn create_daemon(
    log_dir: Option<PathBuf>,
) -> Result<Daemon<ManagementInterfaceEventBroadcaster>, String> {
    let resource_dir = mullvad_paths::get_resource_dir();
    let settings_dir = mullvad_paths::settings_dir()
        .map_err(|e| e.display_chain_with_msg("Unable to get settings dir"))?;
    let cache_dir = mullvad_paths::cache_dir()
        .map_err(|e| e.display_chain_with_msg("Unable to get cache dir"))?;

    let command_channel = DaemonCommandChannel::new();
    let event_listener = spawn_management_interface(command_channel.sender())?;

    Daemon::start(
        log_dir,
        resource_dir,
        settings_dir,
        cache_dir,
        event_listener,
        command_channel,
    )
    .await
    .map_err(|e| e.display_chain_with_msg("Unable to initialize daemon"))
}

fn spawn_management_interface(
    command_sender: DaemonCommandSender,
) -> Result<ManagementInterfaceEventBroadcaster, String> {
    let (socket_path, event_broadcaster) = ManagementInterfaceServer::start(command_sender)
        .map_err(|error| {
            error.display_chain_with_msg("Unable to start management interface server")
        })?;

    log::info!("Management interface listening on {}", socket_path);

    Ok(event_broadcaster)
}

#[cfg(unix)]
fn running_as_admin() -> bool {
    let uid = unsafe { libc::getuid() };
    uid == 0
}

#[cfg(windows)]
fn running_as_admin() -> bool {
    // TODO: Check if user is administrator correctly on Windows.
    true
}
