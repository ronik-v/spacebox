use rand::{RngCore, rngs::OsRng};
use base64::{Engine as _, engine::general_purpose};


pub fn get_auth_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}