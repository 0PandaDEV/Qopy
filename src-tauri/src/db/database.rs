use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::fs;
use tauri::Manager;
use tokio::runtime::Runtime as TokioRuntime;

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let rt = TokioRuntime::new().expect("Failed to create Tokio runtime");

    let app_data_dir = app.path().app_data_dir().unwrap();
    fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    let db_path = app_data_dir.join("data.db");
    let is_new_db = !db_path.exists();
    if is_new_db {
        fs::File::create(&db_path).expect("Failed to create database file");
    }

    let db_url = format!("sqlite:{}", db_path.to_str().unwrap());
    let pool = rt.block_on(async {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(async {
        apply_schema(&pool).await?;
        if is_new_db {
            if let Err(e) = super::history::initialize_history(&pool).await {
                eprintln!("Failed to initialize history: {}", e);
            }
            if let Err(e) = super::settings::initialize_settings(&pool).await {
                eprintln!("Failed to initialize settings: {}", e);
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    app.manage(pool);
    app.manage(rt);

    Ok(())
}

async fn apply_schema(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let schema = include_str!("scheme.sql");
    
    let statements: Vec<&str> = schema
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    for statement in statements {
        sqlx::query(statement)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to execute schema statement: {}", e))?;
    }

    Ok(())
}
