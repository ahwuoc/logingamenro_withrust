use super::message::Message;
use super::service::Service;
use super::session::Session;
use crate::command::Command;
use crate::config::Config;
use crate::db::DbManager;
use crate::model::user::User;
use crate::model::user_manager::UserManager;
use anyhow::Result;
use chrono::Utc;

pub struct Controller {
    db: DbManager,
    user_manager: UserManager,
    config: Config,
}

impl Controller {
    pub fn new(db: DbManager, user_manager: UserManager, config: Config) -> Self {
        Self {
            db,
            user_manager,
            config,
        }
    }
    pub async fn process(&self, session: &mut Session, msg: Message) -> Result<()> {
        match msg.command {
            Command::LOGIN => self.login(session, msg).await?,
            Command::LOGOUT => self.logout(session, msg).await?,
            Command::SET_SERVER => self.set_server(session, msg).await?,
            _ => println!("Unknown command: {}", msg.command),
        }
        Ok(())
    }

    async fn login(&self, session: &mut Session, mut msg: Message) -> Result<()> {
        let server_id = msg.read_byte()?;
        let client_id = msg.read_int()?;
        let username = msg.read_utf()?;
        let password = msg.read_utf()?;

        println!("Login username: {} serverID: {}", username, server_id);

        match User::find_by_credentials(self.db.get_pool(), &username, &password).await {
            Ok(Some(user)) => {
                if user.server_login != server_id as i32 {
                    let msg = format!("Account nay thuoc may chu SV{}", user.server_login);
                    Service::login_failed(session, client_id, &msg).await?;
                    return Ok(());
                }
                if self.user_manager.is_online(user.id).await {
                    Service::disconnect(session, user.id).await?;
                    self.user_manager.remove(user.id).await;
                    Service::login_failed(
                        session,
                        client_id,
                        "Đăng nhập thất bại, vui lòng đăng nhập lại!",
                    )
                    .await?;
                    return Ok(());
                }

                // Check 3: Thời gian chờ giữa các lần login
                let now = Utc::now().timestamp_millis();
                let last_logout = user.last_time_logout.timestamp_millis();
                let seconds_pass = ((now - last_logout) / 1000) as i32;
                let wait_login = self.config.server.second_wait_login;

                if seconds_pass < wait_login {
                    let msg = format!(
                        "Vui lòng chờ {} giây để đăng nhập lại.",
                        wait_login - seconds_pass
                    );
                    Service::login_failed(session, client_id, &msg).await?;
                    return Ok(());
                }

                // Check 4: Testmode (chỉ admin mới login được)
                if !user.is_admin && self.config.server.testmode == 1 {
                    Service::login_failed(
                        session,
                        client_id,
                        "Server đang được admin xử lý và kiểm tra lại,vui lòng quay lại sau",
                    )
                    .await?;
                    return Ok(());
                }
                if user.ban {
                    Service::login_failed(
                        session,
                        client_id,
                        "Tài khoản đã bị khóa do vi phạm điều khoản!",
                    )
                    .await?;
                    return Ok(());
                }

                User::update_login_time(self.db.get_pool(), user.id).await?;
                Service::login_successful(session, &user, client_id).await?;
                self.user_manager
                    .add(user.id, username.clone(), server_id as i32, client_id)
                    .await;
                println!("User {} logged in successfully", username);
            }
            Ok(None) => {
                Service::login_failed(
                    session,
                    client_id,
                    "Thông tin tài khoản hoặc mật khẩu không chính xác",
                )
                .await?;
                println!("Login failed for user: {}", username);
            }
            Err(e) => {
                eprintln!("Database error during login: {}", e);
                Service::login_failed(session, client_id, "Lỗi hệ thống, vui lòng thử lại!")
                    .await?;
            }
        }
        Ok(())
    }
    async fn logout(&self, _session: &mut Session, mut msg: Message) -> Result<()> {
        let user_id = msg.read_int()?;
        if let Some(user_info) = self.user_manager.find(user_id).await {
            println!("Logout user: {}", user_info.username);

            if let Err(e) = User::update_logout_time(self.db.get_pool(), user_id).await {
                eprintln!("Failed to update logout time: {}", e);
            }
            self.user_manager.remove(user_id).await;
        }

        Ok(())
    }
    async fn set_server(&self, _session: &mut Session, mut msg: Message) -> Result<()> {
        let server_id = msg.read_int()?;
        self.user_manager.remove_all_with_server_id(server_id).await;

        let size = msg.read_int()?;
        for i in 0..size {
            let client_id = msg.read_int()?;
            let user_id = msg.read_int()?;
            let username = msg.read_utf()?;
            let _password = msg.read_utf()?;
            println!("  [{}] Add user: {} (id: {})", i + 1, username, user_id);
            self.user_manager
                .add(user_id, username, server_id, client_id)
                .await;
        }
        println!("Server sync completed");
        Ok(())
    }
}
