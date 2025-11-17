use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// UserManager để track users đang online (giống Java version)
#[derive(Clone)]
pub struct UserManager {
    users: Arc<RwLock<HashMap<i32, UserInfo>>>,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: i32,
    pub username: String,
    pub server_id: i32,
    pub client_id: i32,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Thêm user vào danh sách online
    pub async fn add(&self, user_id: i32, username: String, server_id: i32, client_id: i32) {
        let mut users = self.users.write().await;
        users.insert(
            user_id,
            UserInfo {
                user_id,
                username,
                server_id,
                client_id,
            },
        );
    }

    /// Xóa user khỏi danh sách online
    pub async fn remove(&self, user_id: i32) {
        let mut users = self.users.write().await;
        users.remove(&user_id);
    }

    /// Tìm user theo ID
    pub async fn find(&self, user_id: i32) -> Option<UserInfo> {
        let users = self.users.read().await;
        users.get(&user_id).cloned()
    }

    /// Xóa tất cả users của một server
    pub async fn remove_all_with_server_id(&self, server_id: i32) {
        let mut users = self.users.write().await;
        users.retain(|_, user| user.server_id != server_id);
    }

    /// Kiểm tra user có đang online không
    pub async fn is_online(&self, user_id: i32) -> bool {
        let users = self.users.read().await;
        users.contains_key(&user_id)
    }

    /// Đếm số users online
    pub async fn count(&self) -> usize {
        let users = self.users.read().await;
        users.len()
    }
}
