use include_dir::{ include_dir, Dir };
use sqlx::sqlite::{ SqlitePool, SqlitePoolOptions };
use std::fs;
use tauri::Manager;
use tokio::runtime::Runtime as TokioRuntime;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/db/migrations");

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let rt = TokioRuntime::new().expect("Failed to create Tokio runtime");
    app.manage(rt);

    let rt = app.state::<TokioRuntime>();

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
            .connect(&db_url).await
            .expect("Failed to create pool")
    });

    app.manage(pool.clone());

    rt.block_on(async {
        apply_migrations(&pool).await?;
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

    Ok(())
}

async fn apply_migrations(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx
        ::query(
            "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
        )
        .execute(pool).await?;

    let current_version: Option<i64> = sqlx
        ::query_scalar("SELECT MAX(version) FROM schema_version")
        .fetch_one(pool).await?;

    let current_version = current_version.unwrap_or(0);

    let mut migration_files: Vec<(i64, &str)> = MIGRATIONS_DIR.files()
        .filter_map(|file| {
            let file_name = file.path().file_name()?.to_str()?;
            if file_name.ends_with(".sql") && file_name.starts_with("v") {
                let version: i64 = file_name
                    .trim_start_matches("v")
                    .trim_end_matches(".sql")
                    .parse()
                    .ok()?;
                Some((version, file.contents_utf8()?))
            } else {
                None
            }
        })
        .collect();

    migration_files.sort_by_key(|(version, _)| *version);

    for (version, content) in migration_files {
        if version > current_version {
            let statements: Vec<&str> = content
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            for statement in statements {
                sqlx
                    ::query(statement)
                    .execute(pool).await
                    .map_err(|e| format!("Failed to execute migration {}: {}", version, e))?;
            }

            sqlx
                ::query("INSERT INTO schema_version (version) VALUES (?)")
                .bind(version)
                .execute(pool).await?;
        }
    }

    Ok(())
}
