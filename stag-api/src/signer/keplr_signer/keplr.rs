use anyhow::{anyhow, Context, Result};
use js_sys::{Error as JsError, Uint8Array};
use k256::ecdsa::VerifyingKey;
use wasm_bindgen::{prelude::*, JsCast};

use crate::types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKey};

#[wasm_bindgen]
extern "C" {
    type JsKeplr;

    type JsKeplrKey;

    type JsKeplrSignature;

    #[wasm_bindgen(js_name = enable, method, catch)]
    async fn enable(this: &JsKeplr, chain_id: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(js_name = getKey, method, catch)]
    async fn get_key(this: &JsKeplr, chain_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = signArbitrary, method, catch)]
    async fn sign_arbitrary(
        this: &JsKeplr,
        chain_id: &str,
        account_address: &str,
        bytes: Uint8Array,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = pubKey, method, getter)]
    fn get_public_key(this: &JsKeplrKey) -> Uint8Array;

    #[wasm_bindgen(js_name = bech32Address, method, getter)]
    fn to_account_address(this: &JsKeplrKey) -> String;

    #[wasm_bindgen(js_name = signature, method, getter)]
    fn signature(this: &JsKeplrSignature) -> String;
}

pub struct Keplr {
    inner: JsKeplr,
}

impl Keplr {
    pub fn get() -> Option<Keplr> {
        let inner = web_sys::window()?.get("keplr")?.unchecked_into();
        Some(Self { inner })
    }

    pub async fn enable(&self, chain_id: &ChainId) -> Result<()> {
        self.inner
            .enable(&chain_id.to_string())
            .await
            .map_err(to_anyhow_error)
    }

    pub async fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey> {
        self.enable(chain_id).await?;
        let key: JsKeplrKey = self
            .inner
            .get_key(&chain_id.to_string())
            .await
            .map_err(to_anyhow_error)?
            .unchecked_into();

        let bytes = key.get_public_key().to_vec();

        let verifying_key = VerifyingKey::from_sec1_bytes(&bytes)?;

        Ok(PublicKey::Secp256k1(verifying_key))
    }

    pub async fn to_account_address(&self, chain_id: &ChainId) -> Result<String> {
        self.enable(chain_id).await?;
        let key: JsKeplrKey = self
            .inner
            .get_key(&chain_id.to_string())
            .await
            .map_err(to_anyhow_error)?
            .unchecked_into();

        Ok(key.to_account_address())
    }

    pub async fn sign(&self, chain_id: &ChainId, message: &[u8]) -> Result<Vec<u8>> {
        let account_address = self.to_account_address(chain_id).await?;

        let signature: JsKeplrSignature = self
            .inner
            .sign_arbitrary(
                &chain_id.to_string(),
                &account_address,
                Uint8Array::from(message),
            )
            .await
            .map_err(to_anyhow_error)?
            .unchecked_into();

        let signature_str = signature.signature();

        base64::decode(&signature_str).context("failed to decode signature")
    }
}

fn to_anyhow_error(js_value: JsValue) -> anyhow::Error {
    anyhow!(
        "JS error: {}",
        ToString::to_string(&JsError::from(js_value).to_string())
    )
}
