use crate::{db, state::State};

pub mod endpoints;

pub async fn init_admin_user(state: &State) -> Result<(), eyre::Error> {
    let config = &state.config;
    let mut connection = state.database_pool.acquire().await?;
    db::user::new_admin_account(&mut connection, &config.admin_user, &config.admin_pwd).await?;
    Ok(())
}
