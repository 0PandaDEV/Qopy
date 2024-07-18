use tauri::{AppHandle, async_runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;


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
                &format!("New Version: {new_ver}\nCurrent Version: {cur_ver}\n\n"),
                "Would you like to install it now?",
            ]);

            app.dialog()
                .message(msg)
                .title("Update Available")
                .ok_button_label("Yes")
                .cancel_button_label("No")
                .show(move |response| {
                    if !response {
                        return;
                    }
                    async_runtime::spawn(async move {
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