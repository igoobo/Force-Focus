use tauri::{command, State};
use crate::ai::model_update::ModelUpdateManager;

#[command]
pub async fn check_model_update(
    token: String,
    manager: State<'_, ModelUpdateManager>, 
) -> Result<bool, String> {
    println!("🖱️ [Command] Manual update requested.");
    manager.check_and_update(&token).await
}
