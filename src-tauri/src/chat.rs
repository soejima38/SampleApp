// パクリ元
// https://github.com/libp2p/rust-libp2p/tree/master/examples/chat

use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    time::Duration,
};

use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::{io, select};
use tracing_subscriber::EnvFilter;
use tauri::{AppHandle, Emitter};



// GossipsubとMdnsを組み合わせたカスタムネットワークビヘイビアを作成します。
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

use tokio::sync::mpsc;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static CHAT_SENDER: Lazy<Mutex<Option<mpsc::UnboundedSender<String>>>> = Lazy::new(|| Mutex::new(None));


#[tauri::command]
pub fn send_chat_message(message: &str){
    println!("send_chat_message: {}", message);
    let sender_guard = CHAT_SENDER.lock().unwrap();
    if let Some(sender) = sender_guard.as_ref() {
        if let Err(e) = sender.send(message.to_string()) {
            println!("send_chat_message error: {:?}", e);
        }
    } else {
        println!("send_chat_message: sender not initialized");
    }
}


pub fn setup_chat_listener(app: AppHandle) {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    {
        let mut sender_guard = CHAT_SENDER.lock().unwrap();
        *sender_guard = Some(tx);
    }
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_chat_listener(app, &mut rx).await {
            eprintln!("Error in start_chat_listener: {}", e);
        }
    });
}

pub async fn start_chat_listener(app: AppHandle, rx: &mut mpsc::UnboundedReceiver<String>) -> Result<(), Box<dyn Error>> {
    // ログの初期化
    let _ = tracing_subscriber::fmt()   
        .with_env_filter(EnvFilter::from_default_env()) 
        .try_init(); 

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            // メッセージをコンテンツアドレス化するため、メッセージのハッシュ値をIDとして使用します。
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // カスタムgossipsub設定を作成
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10)) // デバッグ用にハートビート間隔を長めに設定（ログが多くならないように）
                .validation_mode(gossipsub::ValidationMode::Strict) // メッセージ検証モードをStrictに設定（デフォルトはStrictで署名を強制）
                .message_id_fn(message_id_fn) // コンテンツアドレス化されたメッセージ。同じ内容のメッセージは複数回伝播しない。
                .build()
                .map_err(io::Error::other)?; // buildが適切なstd::error::Errorを返さないための一時的な対応。

            // gossipsubネットワークビヘイビアを構築
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(MyBehaviour { gossipsub, mdns })
        })?
        .build();

    // Gossipsubトピックを作成
    let topic = gossipsub::IdentTopic::new("test-net");
    // トピックにサブスクライブ
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // すべてのインターフェースで、OSが割り当てたポートでリッスン開始
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("STDINからメッセージを入力すると、Gossipsubで接続中のピアに送信されます");

    // メインループ開始
    loop {
        select! {
            Some(msg) = rx.recv() => {
                // フロントエンドや他のスレッドから受信したメッセージをpublish
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), msg.as_bytes()) {
                    app.emit("chat-message", format!("Publish error: {:?}", e)).unwrap();
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    // mDNSで新しいピアを発見
                    for (peer_id, _multiaddr) in list {
                        println!("mDNSで新しいピアを発見: {peer_id}");
                        app.emit("chat-message", format!("mDNSで新しいピアを発見: {}", peer_id)).unwrap();
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    // mDNSでピアの有効期限切れ
                    for (peer_id, _multiaddr) in list {
                        println!("mDNSでピアの有効期限切れ: {peer_id}");
                        app.emit("chat-message", format!("mDNSでピアの有効期限切れ: {}", peer_id)).unwrap();
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => app.emit("chat-message", format!("メッセージ受信: '{}' (id: {}, ピア: {})", String::from_utf8_lossy(&message.data), id, peer_id)).unwrap(),
                SwarmEvent::NewListenAddr { address, .. } => {
                    // 新しいアドレスでリッスン開始
                    println!("ローカルノードがリッスン中: {address}");
                    app.emit("chat-message", format!("ローカルノードがリッスン中: {:?}", address)).unwrap();
                }
                _ => {}
            }
        }
    }
}

