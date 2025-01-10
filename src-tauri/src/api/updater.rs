use tauri::{ async_runtime, AppHandle };
use tauri_plugin_dialog::{ DialogExt, MessageDialogButtons, MessageDialogKind };
use tauri_plugin_updater::UpdaterExt;

pub async fn check_for_updates(app: AppHandle, prompted: bool) {
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

            let window = app.get_webview_window("main").unwrap();
            window.show().unwrap();
            window.set_focus().unwrap();

            app.dialog()
                .message(msg)
                .title("Qopy Update Available")
                .buttons(
                    MessageDialogButtons::OkCancelCustom(
                        String::from("Install"),
                        String::from("Cancel")
                    )
                )
                .show(move |response| {
                    if !response {
                        return;
                    }
                    async_runtime::spawn(async move {
                        match
                            update.download_and_install(
                                |_, _| {},
                                || {}
                            ).await
                        {
                            Ok(_) => {
                                app.dialog()
                                    .message(
                                        "Update installed successfully. The application needs to restart to apply the changes."
                                    )
                                    .title("Qopy Update Installed")
                                    .buttons(
                                        MessageDialogButtons::OkCancelCustom(
                                            String::from("Restart"),
                                            String::from("Cancel")
                                        )
                                    )
                                    .show(move |response| {
                                        if response {
                                            app.restart();
                                        }
                                    });
                            }
                            Err(e) => {
                                println!("Error installing new update: {:?}", e);
                                app.dialog()
                                    .message(
                                        "Failed to install new update. The new update can be downloaded from Github"
                                    )
                                    .kind(MessageDialogKind::Error)
                                    .show(|_| {});
                            }
                        }
                    });
                });
        }
        Ok(None) => {
            println!("No updates available.");
        }
        Err(e) => {
            if prompted {
                let window = app.get_webview_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();

                app.dialog()
                    .message("No updates available.")
                    .title("Qopy Update Check")
                    .show(|_| {});
            }

            println!("No updates available. {}", e.to_string());
        }
    }
}
