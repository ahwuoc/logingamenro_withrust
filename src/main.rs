use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{debug, error, info, trace};

mod command;
mod config;
mod db;
mod io;
mod model;

use config::Config;
use db::DbManager;
use io::controller::Controller;
use io::session::Session;
use model::user_manager::UserManager;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load("config.toml")?;
    info!("Configuration loaded");
    debug!("Config: {:#?}", config);
    trace!("Server listen_port: {}", config.server.listen_port);
    trace!("Server second_wait_login: {}", config.server.second_wait_login);
    trace!("Server testmode: {}", config.server.testmode);
    
    println!("Server Port: {}, Test Mode: {}, Wait Login: {}s", 
        config.server.listen_port,
        config.server.testmode,
        config.server.second_wait_login
    );
    
    println!("Database: {}:{}/{} (user: {}, pool: {}-{})",
        config.database.host,
        config.database.port,
        config.database.database_name,
        config.database.username,
        config.database.min_connections,
        config.database.max_connections
    );
    let db = DbManager::new(&config.database).await?;
    info!("Database connected");

    let user_manager = UserManager::new();

    let addr = format!("0.0.0.0:{}", config.server.listen_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on port: {}", config.server.listen_port);
    println!("@Author dev:Ahwuocdz");
    let mut session_id = 0;
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Client {} connected !", &addr);
                let db_clone = db.clone();
                let user_manager_clone = user_manager.clone();
                let config_clone = config.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_session(
                        stream,
                        session_id,
                        db_clone,
                        user_manager_clone,
                        config_clone,
                    )
                    .await
                    {
                        error!("Session error: {}", e);
                    };
                });
                session_id += 1;
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }

    Ok(())
}
async fn handle_session(
    stream: tokio::net::TcpStream,
    id: i32,
    db: DbManager,
    user_manager: UserManager,
    config: Config,
) -> Result<()> {
    let mut session = Session::new(stream, id);
    let controller = Controller::new(db, user_manager, config);

    while session.is_connected() {
        match session.read_message().await {
            Ok(Some(msg)) => {
                if msg.command == -27 {
                    info!("Game Server requested encryption key");
                    session.send_key().await?;
                    continue;
                }
                controller.process(&mut session, msg).await?;
            }
            Ok(None) => {
                info!("Connection closed by client");
                break;
            }
            Err(e) => {
                error!("Read message error: {}", e);
                break;
            }
        }
    }
    session.close();
    info!("Session {} disconnected", id);
    Ok(())
}
