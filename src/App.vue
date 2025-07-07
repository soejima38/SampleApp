<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}

// イベントを受け取る際に必要なインポート
import { listen } from '@tauri-apps/api/event';

// ダウンロードの進捗を表示するための変数
const eventValue = ref("");

// バックエンドからの「download-started」イベントを受け取る
// 受け取ったときは、a.valueに「downloading from [URL]」と表示する
listen('download-started', (event) => {
  eventValue.value = `downloading from ${event.payload}`
});

  // downsloadコマンドを呼び出す
  // 呼び出すときは第1引数にコマンド名を指定し、
  // 第2引数にオブジェクトを指定する
async function download() {
  await invoke("download", {url: "https://example.com/file.zip"});
}


const startMsg = ref("");   // pint受け取りを開始した際のメッセージを格納する変数
const pintReceived = ref("test2");  // pingを受け取った時のメッセージを格納する変数
async function startWaitPing() {
  // pingの受け取りを開始するメソッド
  await invoke("start_wait_ping");
}

listen('listen-started', (event) => {
  // pingイベントを受け取ったときの処理
  // event.payloadには受け取ったメッセージが入る
  startMsg.value += `\n${event.payload}`;
});

listen('ping-received', (event) => {
  // pingイベントを受け取ったときの処理
  // event.payloadには受け取ったメッセージが入る
  pintReceived.value = `${event.payload}`;
});


// チャット機能のための変数とメソッド
const chatLog = ref("");

listen('chat-message', (event) => {
  // チャットメッセージを受け取ったときの処理
  // event.payloadには受け取ったメッセージが入る
  chatLog.value += `${event.payload}\n`;
});

const message = ref(""); // チャットメッセージを格納する変数

async function sendMessage() {
  // チャットメッセージを送信するメソッド
  // ここでは、バックエンドのチャット機能にメッセージを送信する
  if (!message.value.trim()) {
    return; // 空のメッセージは送信しない
  }
  const msg = message.value.trim();
  message.value = "てｓｔ"; // メッセージを送信した後は入力欄
  
  await invoke("send_chat_message", { message: msg }); // バックエンドのsend_chat_messageコマンドを呼び出す
  chatLog.value += `You: ${msg}\n`; // ログに送信したメッセージを追加
}

</script>

<template>
  <main class="container">
    <div class="chat-container">
        <!-- ログ表示部（複数行テキストボックス/readonly） -->
        <textarea class="chat-log" readonly>{{ chatLog }}</textarea>
        <!-- 入力エリア -->
        <form class="chat-input-area" @submit.prevent="sendMessage">
          <input type="text" v-model="message" placeholder="メッセージを入力...">
          <button type="submit">送信</button>
        </form>
      </div>
    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <button type="submit">Greet</button>
    </form>
    <p>{{ greetMsg }}</p>
  
    <!-- バックエンドからのイベントを取得し、表示するテスト -->
    <form class="test" @submit.prevent="download">  <!-- classはただの名前付け。ラベル。省略できる-->
      <button type="submit">Download</button>       <!-- download関数を呼び出すボタン -->
    </form>
    <p>{{ eventValue }}</p>                         <!-- 受け取ったイベントの内容を表示するための変数を表示 -->

    <!-- pingテスト-->
     <form @submit.prevent="startWaitPing">
      <button type="submit">Ping Start</button>       <!-- download関数を呼び出すボタン -->
    </form>
    <pre>{{ startMsg }}</pre>                         <!-- ping受付開始時のメッセージを表示 -->
    <p>{{ pintReceived }}</p>                         <!-- ping受け取り時のメッセージを表示 -->
  </main>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>

<!-- チャット部のスタイル -->
<style>
    body {
      margin: 0;
      font-family: 'Segoe UI', Arial, sans-serif;
      display: flex;
      flex-direction: column;
      min-height: 100vh;
    }
    .chat-container {
      width: 400px;
      margin: 40px auto;
      background: #fff;
      border-radius: 10px;
      box-shadow: 0 2px 10px #0001;
      display: flex;
      flex-direction: column;
      padding: 18px 18px 0 18px;
    }
    .chat-log {
      width: 95%;
      height: 250px;
      resize: none;
      padding: 10px;
      border: 1px solid #ccc;
      border-radius: 6px;
      font-size: 16px;
      color: #222;
      background: #f9f9f9;
      margin-bottom: 14px;
      overflow-y: auto;
    }
    .chat-input-area {
      display: flex;
      padding-bottom: 18px;
      gap: 8px;
    }
    .chat-input-area input {
      flex: 1;
      padding: 8px 12px;
      border: 1px solid #bbb;
      border-radius: 4px;
      font-size: 16px;
    }
    .chat-input-area button {
      padding: 8px 16px;
      background: #24c8db;
      color: #fff;
      border: none;
      border-radius: 4px;
      font-size: 16px;
      cursor: pointer;
    }
    .chat-input-area button:hover {
      background: #189aa8;
    }
  </style>