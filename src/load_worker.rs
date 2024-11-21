use reqwest::Url;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub async fn load_bytes(origin: String, asset: String) -> Vec<u8> {
    let url = Url::parse(&format!("{}/assets/{}", origin, asset)).unwrap();

    reqwest::get(url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec()
}
