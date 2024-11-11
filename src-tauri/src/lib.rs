// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod components;
use chrono::{Duration, Local};
use components::db::{get_instance, get_last_user, get_user_by_id, set_user_state, AppState};
use components::monitor::{compare_images, hook_msg, LiveUser, Message};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
async fn add_user(
    name: &str,
    url: &str,
    hook: &str,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    match sqlx::query("INSERT INTO users (name, url, hook) VALUES (?,?,?)")
        .bind(name)
        .bind(url)
        .bind(hook)
        .execute(&state.pool)
        .await
    {
        Ok(row) => {
            state
                .max_id
                .store(row.last_insert_rowid() as i32, Ordering::SeqCst);

            Ok(format!("已添加商家: {}, 直播地址: {}", name, url))
        }
        Err(e) => Err(format!("add user error: {}", e)),
    }
}

#[tauri::command]
async fn get_all_user(state: State<'_, Arc<AppState>>) -> Result<Vec<LiveUser>, String> {
    let query_str =
        "SELECT id, name, url, hook, status, created_at, updated_at FROM users ORDER BY id DESC";
    match sqlx::query_as::<_, LiveUser>(query_str)
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => {
            match rows.first() {
                Some(row) => {
                    state.max_id.store(row.id, Ordering::SeqCst);
                    // println!("max_id: {}", row.id);
                }
                None => {
                    println!("empty database!");
                }
            }
            Ok(rows)
        }
        Err(e) => Err(format!("Error fetching users: {}", e)),
    }
}

#[tauri::command]
async fn get_next_user(state: State<'_, Arc<AppState>>) -> Result<Option<LiveUser>, String> {
    let current_id = match state.current_id.load(Ordering::SeqCst) {
        n if n < state.max_id.load(Ordering::SeqCst) => n,
        _ => -1
    };
    // println!("next_id : {}", next_id);
    // println!("max_id: {}", state.max_id.load(Ordering::SeqCst));
    // println!("current_id: {}", state.current_id.load(Ordering::SeqCst));
    let query_str: String = format!(
            "SELECT id, name, url, hook, status, created_at, updated_at FROM users WHERE id>{} ORDER BY id ASC",
           current_id 
        );
    match sqlx::query_as::<_, LiveUser>(&query_str)
        .fetch_one(&state.pool)
        .await
    {
        Ok(row) => {
            state.current_id.store(row.id, Ordering::SeqCst);
            println!("current_id: {}", state.current_id.load(Ordering::SeqCst));
            Ok(Some(row))
        }
        Err(e) => Err(format!("Error fetching users: {}", e)),
    }
}

#[tauri::command]
async fn analysis(state: State<'_, Arc<AppState>>) -> Result<i32, ()> {
    let current_id = state.current_id.load(Ordering::SeqCst);
    if current_id != -1 {
        if let Some(current_user) = get_user_by_id(current_id, &state.pool).await {
            // println!("analysis current id: {}", current_id);
            // println!("analysis current user: {}", current_user.name.clone());
            let threshold_time = Local::now() - Duration::hours(2);
            let is_different = compare_images();
            if is_different {
                println!("different images !");
                if !current_user.status {
                    let msg = Message {
                        name: current_user.name,
                        url: current_user.url,
                        updated_at: current_user.updated_at,
                        desp: String::from("直播画面恢复正常!"),
                    };
                    match hook_msg(msg, current_user.hook).await {
                        Ok(()) => {
                            println!("send hook msg success");
                        }
                        Err(e) => {
                            eprintln!("send hook msg failed: {}", e);
                        }
                    }
                }
            } else {
                println!("same images!");

                // send msg every 2 hours
                if current_user.status || current_user.updated_at < threshold_time.naive_local() {
                    let msg = Message {
                        name: current_user.name,
                        url: current_user.url,
                        updated_at: current_user.updated_at,
                        desp: String::from("直播画面相似度太高，可能异常!"),
                    };
                    match hook_msg(msg, current_user.hook).await {
                        Ok(()) => {
                            println!("send hook msg success");
                        }
                        Err(e) => {
                            eprintln!("send hook msg failed: {}", e);
                        }
                    }
                }
            }
            set_user_state(current_user.id, is_different, &state.pool).await;
        }
    }
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

    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default().plugin(tauri_plugin_devtools::init());

    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();

    builder
        .manage(state.clone())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            add_user,
            get_all_user,
            get_next_user,
            analysis
        ])
        .run(tauri::generate_context!())
        .expect("error while running nnklivemonitor application");
}
