use aes_gcm::{ Aes256Gcm, KeyInit };
use aes_gcm::aead::Aead;
use base64::{ engine::general_purpose::STANDARD, Engine };
use rand::{ Rng, thread_rng };
use rand::seq::SliceRandom;
use serde::{ Deserialize, Serialize };
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
        let mut rng = thread_rng();
        let pairing_key = Self::generate_emoji_sequence(&mut rng);
        let encryption_key: [u8; 32] = rng.gen();
        let nonce: [u8; 12] = rng.gen();
        PairingManager {
            pairing_key,
            encryption_key,
            nonce,
        }
    }

    pub fn generate_emoji_sequence(rng: &mut impl Rng) -> String {
        let key: Vec<&str> = EMOJI_POOL.choose_multiple(rng, PAIRING_KEY_LENGTH).cloned().collect();
        key.join(" ")
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

    pub fn generate_invitation_code(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn encrypt_key(&self, key: &[u8; 32]) -> Result<String, String> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let ciphertext = cipher
            .encrypt(&self.nonce.into(), key.as_ref())
            .map_err(|e| e.to_string())?;
        Ok(STANDARD.encode(ciphertext))
    }

    pub fn decrypt_key(&self, encrypted_key: &str) -> Result<[u8; 32], String> {
        let ciphertext = STANDARD.decode(encrypted_key).map_err(|e| e.to_string())?;
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let plaintext = cipher
            .decrypt(&self.nonce.into(), ciphertext.as_ref())
            .map_err(|e| e.to_string())?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&plaintext);
        Ok(key)
    }

    pub fn create_pairing_request(&self, inviter_id: String) -> PairingRequest {
        PairingRequest {
            inviter_id,
            invitation_code: self.generate_invitation_code(),
        }
    }

    pub fn handle_pairing_response(&self, response: PairingRequest) -> bool {
        self.validate_pairing(&response.invitation_code)
    }
}

#[tauri::command]
pub fn initiate_pairing(_pairing_manager: State<'_, PairingManager>) -> String {
    let mut rng = thread_rng();
    PairingManager::generate_emoji_sequence(&mut rng)
}

#[tauri::command]
pub fn complete_pairing(
    input_key: String,
    pairing_manager: State<'_, PairingManager>
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
    pairing_manager.generate_invitation_code()
}
