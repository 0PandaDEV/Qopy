use tauri::plugin::TauriPlugin;
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;
use tokio;

pub fn init() -> TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("updater")
        .invoke_handler(tauri::generate_handler![check_for_updates])
        .build()
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) {
    println!("Checking for updates...");

    let updater = app.updater().unwrap();
    let response = updater.check().await;

    match response {
        Ok(Some(update)) => {
            let cur_ver = &update.current_version;
            let new_ver = &update.version;
            let mut msg = String::new();
            msg.extend([
                &format!("{cur_ver} -> {new_ver}\n\n"),
                "Would you like to install it now?",
            ]);

            app.dialog()
                .message(msg)
                .title("Update Available")
                .ok_button_label("Install")
                .cancel_button_label("Cancel")
                .show(move |response| {
                    if !response {
                        return;
                    }
                    tokio::spawn(async move {
                        if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                            println!("Error installing new update: {:?}", e);
                            app.dialog().message(
                                "Failed to install new update. The new update can be downloaded from Github"
                            ).kind(MessageDialogKind::Error).show(|_| {});
                        }
                    });
                });
        }
        Ok(None) => println!("No updates available."),
        Err(e) => {
            println!("Failed to check for updates: {:?}", e);
        }
    }
}