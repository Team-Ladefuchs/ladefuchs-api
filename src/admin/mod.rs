use crate::{db, state::State};

pub mod api_endpoints;
pub mod auth;

pub async fn init_admin_user(state: &State) -> Result<(), eyre::Error> {
    let config = &state.config;
    let mut connection = state.database_pool.acquire().await?;
    if let (Some(user_name), Some(password)) = (&config.admin_user, &config.admin_pwd) {
        db::user::new_admin_account(&mut connection, user_name, password).await?;
    }
    Ok(())
}
