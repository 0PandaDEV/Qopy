use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::rng;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

const EMOJI_POOL: &[&str] = &["😀", "😁", "😂", "🤣", "😃", "😄", "😅", "😆", "😉", "😊"];
const PAIRING_KEY_LENGTH: usize = 4;

#[derive(Serialize, Deserialize)]
pub struct PairingRequest {
    pub inviter_id: String,
    pub invitation_code: String,
}

pub struct PairingManager {
    pairing_key: String,
    encryption_key: [u8; 32],
    nonce: [u8; 12],
}

impl PairingManager {
    pub fn new() -> Self {
        let mut rng = rng();
        let pairing_key = Self::generate_emoji_sequence(&mut rng);
        let encryption_key = rng.random::<[u8; 32]>();
        let nonce = rng.random::<[u8; 12]>();
        PairingManager {
            pairing_key,
            encryption_key,
            nonce,
        }
    }

    pub fn generate_emoji_sequence<R: Rng>(rng: &mut R) -> String {
        let mut emojis = EMOJI_POOL.to_vec();
        emojis.shuffle(rng);
        emojis
            .iter()
            .take(PAIRING_KEY_LENGTH)
            .cloned()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn validate_pairing(&self, input_key: &str) -> bool {
        self.pairing_key == input_key
    }

    pub fn get_encryption_key(&self) -> &[u8; 32] {
        &self.encryption_key
    }

    pub fn get_nonce(&self) -> &[u8; 12] {
        &self.nonce
    }
}

#[tauri::command]
pub fn generate_invitation_code(_pairing_manager: State<'_, PairingManager>) -> String {
    Uuid::new_v4().to_string()
}

#[tauri::command]
pub fn encrypt_key(
    pairing_manager: State<'_, PairingManager>,
    key: Vec<u8>,
) -> Result<String, String> {
    let key_array: [u8; 32] = key
        .try_into()
        .map_err(|_| "Invalid key length".to_string())?;
    let cipher = Aes256Gcm::new(&pairing_manager.encryption_key.into());
    let ciphertext = cipher
        .encrypt(&pairing_manager.nonce.into(), key_array.as_ref())
        .map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(ciphertext))
}

#[tauri::command]
pub fn decrypt_key(
    pairing_manager: State<'_, PairingManager>,
    encrypted_key: String,
) -> Result<Vec<u8>, String> {
    let ciphertext = STANDARD.decode(&encrypted_key).map_err(|e| e.to_string())?;
    let cipher = Aes256Gcm::new(&pairing_manager.encryption_key.into());
    let plaintext = cipher
        .decrypt(&pairing_manager.nonce.into(), ciphertext.as_ref())
        .map_err(|e| e.to_string())?;
    Ok(plaintext)
}

#[tauri::command]
pub fn create_pairing_request(
    pairing_manager: State<'_, PairingManager>,
    inviter_id: String,
) -> PairingRequest {
    PairingRequest {
        inviter_id,
        invitation_code: generate_invitation_code(pairing_manager),
    }
}

#[tauri::command]
pub fn handle_pairing_response(
    pairing_manager: State<'_, PairingManager>,
    response: PairingRequest,
) -> bool {
    pairing_manager.validate_pairing(&response.invitation_code)
}

#[tauri::command]
pub fn initiate_pairing(_pairing_manager: State<'_, PairingManager>) -> String {
    let mut rng = rng();
    PairingManager::generate_emoji_sequence(&mut rng)
}

#[tauri::command]
pub fn complete_pairing(
    input_key: String,
    pairing_manager: State<'_, PairingManager>,
) -> Result<String, String> {
    if pairing_manager.validate_pairing(&input_key) {
        let _shared_key = pairing_manager.encryption_key.to_vec();
        Ok("Pairing successful".to_string())
    } else {
        Err("Invalid pairing key".to_string())
    }
}

#[tauri::command]
pub fn generate_invitation(pairing_manager: State<'_, PairingManager>) -> String {
    generate_invitation_code(pairing_manager)
}
