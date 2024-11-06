// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod components;
use components::db::{get_instance, get_last_user, AppState};
use components::monitor::{compare_images, LiveUser};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
async fn add_user(
    name: &str,
    url: &str,
    hook: &str,
    state: State<'_, Arc<AppState>>,
) -> Result<String, ()> {
    let _ = sqlx::query("INSERT INTO users (name, url, hook) VALUES (?, ?, ?)")
        .bind(name)
        .bind(url)
        .bind(hook)
        .execute(&state.pool)
        .await;
    Ok(format!("已添加商家: {}, 直播地址: {}", name, url))
}

#[tauri::command]
async fn get_all_user(state: State<'_, Arc<AppState>>) -> Result<Vec<LiveUser>, String> {
    match sqlx::query_as::<_, LiveUser>("SELECT id,name,url,hook FROM users ORDER BY id DESC")
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => Ok(rows),
        Err(e) => Err(format!("Error fetching users: {}", e)),
    }
}

#[tauri::command]
async fn get_current_user(state: State<'_, Arc<AppState>>) -> Result<Option<LiveUser>, String> {
    match sqlx::query_as::<_, LiveUser>(
        format!(
            "SELECT id,name,url,hook FROM users ORDER BY id DESC WHERE id={}",
            state.current_id.load(Ordering::SeqCst)
        )
        .as_str(),
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(rows) => Ok(Some(rows)),
        Err(e) => Err(format!("Error fetching users: {}", e)),
    }
}

#[tauri::command]
async fn get_next_user(state: State<'_, Arc<AppState>>) -> Result<Option<LiveUser>, String> {
    let mut next_id = state.current_id.load(Ordering::SeqCst);
    if next_id > state.max_id.load(Ordering::SeqCst) {
        next_id = -1;
    }
    println!("next_id : {}", next_id);
    println!("max_id: {}", state.max_id.load(Ordering::SeqCst));
    println!("current_id: {}", state.current_id.load(Ordering::SeqCst));
    let query_str: String = match next_id {
        -1 => String::from("SELECT id, name, url,hook FROM users"),
        _ => format!("SELECT id, name, url,hook FROM users WHERE id={}", next_id),
    };
    match sqlx::query_as::<_, LiveUser>(&query_str)
        .fetch_one(&state.pool)
        .await
    {
        Ok(rows) => {
            state.current_id.store(rows.id + 1, Ordering::SeqCst);
            println!("current_id: {}", state.current_id.load(Ordering::SeqCst));
            Ok(Some(rows))
        }
        Err(e) => Err(format!("Error fetching users: {}", e)),
    }
}

#[tauri::command]
async fn analysis(state: State<'_, Arc<AppState>>) -> Result<i32, ()> {
    let current_id = state.current_id.load(Ordering::SeqCst);
    if current_id != -1 {
        if compare_images() {
            println!("different images !");
        } else {
            println!("same images!");
        }
    }
    println!("analysis current id: {}", current_id);
    Ok(current_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pool = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_instance())
        .unwrap();
    let max_user_id = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_last_user(&pool))
    {
        Some(user) => user.id,
        None => 0,
    };
    let state = Arc::new(AppState {
        pool,
        current_id: AtomicI32::new(-1),
        max_id: AtomicI32::new(max_user_id),
    });
    tauri::Builder::default()
        .manage(state.clone())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            add_user,
            get_all_user,
            get_next_user,
            analysis
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
