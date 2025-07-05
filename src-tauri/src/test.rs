// 以下のURLから持ってきたサンプルコード
// https://tauri.app/develop/calling-frontend/
use tauri::{AppHandle, Emitter};


// フロントエンドから呼び出されたら順にメッセージを返すコマンド
// 受け取る側は
// `listen('[通知名(download-started等)]', (event) => { ... })` のように書く
#[tauri::command]
pub  fn download(app: AppHandle, url: String) {
  app.emit("download-started", &url).unwrap();
  for progress in [1, 15, 50, 80, 100] {
    app.emit("download-progress", progress).unwrap();
  }
  app.emit("download-finished", &url).unwrap();
}