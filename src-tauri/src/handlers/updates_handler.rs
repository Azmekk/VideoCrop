use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> bool {
    println!("Starting to check for updates...");

    let update_opt_result = app
        .updater_builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let update_opt_opt = match update_opt_result {
        Ok(updater) => Some(updater.check().await.unwrap_or_else(|e| {
            println!("Failed to check for updates: {}", e);
            None
        })),
        Err(e) => {
            println!("Failed to build updater: {}", e);
            None
        }
    };

    let update_opt = match update_opt_opt {
        Some(update) => update,
        None => return false,
    };

    match update_opt {
        Some(_) => {
            println!("Update available.");
            return true;
        }
        None => {
            println!("No updates found.");
            return false;
        }
    }
}

#[tauri::command]
pub async fn update_app(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    println!("Starting to update app...");
    if let Some(update) = app
        .updater_builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?
        .check()
        .await?
    {
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    println!("download finished");
                },
            )
            .await?;

        println!("update installed");
        app.restart();
    }

    Ok(())
}
