use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use std::sync::Arc;
use typenum::U12;

const KVS_URL: &str = "https://kvs.wireway.ch";

#[derive(Serialize, Deserialize, Clone)]
pub struct ClipData {
    content: String,
    content_type: String,
    timestamp: u64,
}

#[derive(Clone)]
pub struct ClipboardSync {
    client: Client,
    cipher: Aes256Gcm,
    nonce: Nonce<U12>,
}

impl ClipboardSync {
    pub fn new(encryption_key: &[u8; 32], nonce_bytes: &[u8; 12]) -> Self {
        let cipher = Aes256Gcm::new(encryption_key.into());
        let nonce = Nonce::from_slice(nonce_bytes).clone();
        ClipboardSync {
            client: Client::new(),
            cipher,
            nonce,
        }
    }

    pub async fn send_clipboard(&self, clip: ClipData) -> Result<(), String> {
        let plaintext = serde_json::to_string(&clip).map_err(|e| e.to_string())?;
        let ciphertext = self.cipher.encrypt(&self.nonce, plaintext.as_bytes()).map_err(|e| e.to_string())?;
        let encoded = STANDARD.encode(ciphertext);
        self.client
            .post(&format!("{}/clipboard", KVS_URL))
            .json(&serde_json::json!({
                "key": "clipboard",
                "value": encoded,
                "expires_in": 60
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn receive_clipboard(&self, app_handle: AppHandle) -> Result<(), String> {
        let res = self.client.get(&format!("{}/clipboard", KVS_URL)).send().await.map_err(|e| e.to_string())?;
        if res.status().is_success() {
            let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            if let Some(encoded) = json["value"].as_str() {
                let ciphertext = STANDARD.decode(encoded).map_err(|e| e.to_string())?;
                let plaintext = self.cipher.decrypt(&self.nonce, ciphertext.as_ref()).map_err(|e| e.to_string())?;
                let clip_str = String::from_utf8(plaintext).map_err(|e| e.to_string())?;
                let clip: ClipData = serde_json::from_str(&clip_str).map_err(|e| e.to_string())?;
                app_handle.emit("clipboard-update", clip).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub async fn listen_webhook(&self, app_handle: AppHandle, state: Arc<Mutex<Self>>) {
        tokio::spawn(async move {
            loop {
                if let Err(_) = state.lock().await.receive_clipboard(app_handle.clone()).await {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }
}

#[tauri::command]
pub async fn send_clipboard_data(clip: ClipData, sync: tauri::State<'_, Arc<Mutex<ClipboardSync>>>) -> Result<(), String> {
    let sync = sync.lock().await;
    sync.send_clipboard(clip).await
}

#[tauri::command]
pub async fn receive_clipboard_data(app_handle: AppHandle, sync: tauri::State<'_, Arc<Mutex<ClipboardSync>>>) -> Result<(), String> {
    let sync = sync.lock().await;
    sync.receive_clipboard(app_handle).await
}
