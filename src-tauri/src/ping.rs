// このコードをパクった
// https://github.com/libp2p/rust-libp2p/tree/master/examples/ping

use std::{error::Error,  time::Duration};

use futures::prelude::*;
use libp2p::{noise, ping, swarm::SwarmEvent, tcp, yamux, Multiaddr};
use tracing_subscriber::EnvFilter;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn start_wait_ping(app: AppHandle) {
    tauri::async_runtime::spawn(async move {    // 非同期で実行するためにspawnを使用(必須)
        if let Err(e) = start_ping_listener(app).await { // pingの受け取りを開始するメソッドを呼び出す
            eprintln!("Error in tmp_method: {}", e);    // とりあえず動くから放置しているだけで必須かわかっていない
        }
    });
}

async fn start_ping_listener(app: AppHandle)-> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()                   // これはログ出力の設定   
        .with_env_filter(EnvFilter::from_default_env()) // 環境変数でログレベルを設定できるようにする
        .try_init();                                    // ログ出力の初期化

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()   // P2P通信で使う秘密鍵・公開鍵を生成
        .with_tokio()   // Tokioランタイムを使用
        .with_tcp(      // TCPを使用
            tcp::Config::default(), 
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| ping::Behaviour::default())?                                          // ふるまいとしてpingを使用
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))   // 接続のタイムアウトを最大値に設定
        .build();                                                                                   // Swarmをビルド

    // すべてのIPv4アドレス + OSが割り当てるランダムなポートでリッスンを開始
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        match swarm.select_next_some().await {  // イベントを非同期で待つ
            SwarmEvent::NewListenAddr { address, .. } => app.emit("listen-started", format!("Listening on {:?}", address)).unwrap(),  // 待ち受けアドレスができたとき
            SwarmEvent::Behaviour(event) => app.emit("ping-received", format!("{:?}", event)).unwrap(),  // pingのイベントを受け取ったとき
            _ => {} // その他のイベントは無視
        }
    }
}
