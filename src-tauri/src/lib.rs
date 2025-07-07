pub mod test;
pub mod ping;
pub mod chat;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,          // フロントエンドから呼び出すコマンドはカンマ区切りで追加可能
            test::download,  // 別ファイルからコマンドを呼ぶときは「pub mod ファイル名;」を書いたうえで「ファイル名::関数名」を指定
            ping::start_wait_ping, // pingの受け取りを開始するメソッド
            chat::send_chat_message, // チャットメッセージを送信するコマンド
            ]) 
        .setup(|app| {
            let handle = app.handle();
            chat::setup_chat_listener(handle.clone()); // チャットのリスナーを開始
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

}
