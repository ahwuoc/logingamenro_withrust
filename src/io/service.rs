use super::message::Message;
use super::session::Session;
use crate::model::user::User;
use anyhow::Result;

pub struct Service;

impl Service {
    pub async fn login_successful(
        session: &mut Session,
        user: &User,
        client_id: i32,
    ) -> Result<()> {
        let mut msg = Message::new(1);
        msg.write_int(client_id);
        msg.write_byte(0);
        msg.write_int(user.id);
        msg.write_bool(user.is_admin);
        msg.write_bool(user.active);
        msg.write_int(user.thoi_vang);
        msg.write_long(user.last_time_login.timestamp_millis());
        msg.write_long(user.last_time_logout.timestamp_millis());

        if let Some(ref rewards) = user.reward {
            msg.write_utf(rewards);
        } else {
            msg.write_utf("");
        }
        msg.write_int(0); // ruby - not used
        msg.write_int(0); // moc_nap - not used
        msg.write_int(user.server_login);
        msg.write_int(0); // is_use_ma_bao_ve - not used
        msg.write_int(0); // ma_bao_ve - not used
        msg.write_int(user.tongnap);
        msg.write_int(user.vnd);
        session.send_message(&msg).await?;
        Ok(())
    }
    pub async fn login_failed(session: &mut Session, client_id: i32, reason: &str) -> Result<()> {
        let mut msg = Message::new(1);
        msg.write_int(client_id);
        msg.write_byte(1);
        msg.write_utf(reason);
        session.send_message(&msg).await?;
        Ok(())
    }
    pub async fn disconnect(session: &mut Session, user_id: i32) -> Result<()> {
        let mut msg = Message::new(3);
        msg.write_int(user_id);
        session.send_message(&msg).await?;
        Ok(())
    }
    pub async fn server_message(session: &mut Session, client_id: i32, text: &str) -> Result<()> {
        let mut msg = Message::new(4);
        msg.write_int(client_id);
        msg.write_utf(text);
        session.send_message(&msg).await?;
        Ok(())
    }
    pub async fn update_time_logout(session: &mut Session, user_id: i32) -> Result<()> {
        let mut msg = Message::new(6);
        msg.write_int(user_id);
        session.send_message(&msg).await?;
        Ok(())
    }
}
