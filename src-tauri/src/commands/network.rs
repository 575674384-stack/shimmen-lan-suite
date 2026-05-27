use tauri::{command, Manager};
use crate::models::User;
use crate::network::peer::{PeerMap, get_online_users};

#[command]
pub fn get_online_users_cmd(
    app_handle: tauri::AppHandle,
) -> Result<Vec<User>, String> {
    if let Some(state) = app_handle.try_state::<PeerMap>() {
        let peers = state.inner();
        Ok(get_online_users(peers))
    } else {
        Ok(vec![])
    }
}
