use std::io::{stdin, Read};

use anyhow::Result;
use log::{debug, error, info, warn};
use owmods_core::{
    alerts::get_warnings,
    config::Config,
    db::LocalDatabase,
    game::launch_game,
    socket::{LogServer, SocketMessage, SocketMessageType},
};
use tokio::{sync::mpsc, try_join};

fn handle_game_log(message: &SocketMessage) {
    let unknown = &"Unknown".to_string();
    let out_message = format!(
        "[{}::{}][{:?}] {}",
        message.sender_name.as_ref().unwrap_or(unknown),
        message.sender_type.as_ref().unwrap_or(unknown),
        message.message_type,
        message.message
    );
    match message.message_type {
        SocketMessageType::Message
        | SocketMessageType::Info
        | SocketMessageType::Success
        | SocketMessageType::Quit => {
            info!("{}", out_message)
        }
        SocketMessageType::Error | SocketMessageType::Fatal => {
            error!("{}", out_message)
        }
        SocketMessageType::Warning => warn!("{}", out_message),
        SocketMessageType::Debug => debug!("{}", out_message),
    }
}

pub async fn start_just_logs(port: &u16) -> Result<()> {
    let server = LogServer::new(*port).await?;
    let (tx, mut rx) = mpsc::channel(32);

    try_join!(server.listen(tx, false), async {
        while let Some(msg) = rx.recv().await {
            handle_game_log(&msg);
        }
        Ok(())
    })?;

    Ok(())
}

pub async fn start_game(local_db: &LocalDatabase, config: &Config, port: &u16) -> Result<()> {
    let names = config.viewed_alerts.iter().map(|n| n.as_str()).collect();
    let warnings = get_warnings(local_db.active().collect(), names);

    let mut config = config.clone();

    for (unique_name, warning) in warnings {
        let start_banner = format!("====== Warning For {} ======", unique_name);
        let end_banner = "=".repeat(start_banner.len());
        warn!(
            "{}\n{}\n\n{}\n{}\nPress Enter To Continue...",
            start_banner, warning.title, warning.body, end_banner
        );
        stdin().read_exact(&mut [0])?;
        config.set_warning_shown(unique_name);
    }

    config.save()?;

    let server = LogServer::new(*port).await?;
    let port = server.port;

    let (tx, mut rx) = mpsc::channel(32);

    try_join!(
        server.listen(tx, true),
        launch_game(&config, &port),
        async {
            while let Some(msg) = rx.recv().await {
                handle_game_log(&msg);
            }
            Ok(())
        }
    )?;

    Ok(())
}
