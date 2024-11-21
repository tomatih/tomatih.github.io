importScripts("./pkg/personal_page.js")

const {load_bytes} = wasm_bindgen;

let back_buffer = ""; // needed for messages received before WASM initializes

async function init_worker() {

    self.onmessage = async event => {
        back_buffer = event.data;
    }

    await wasm_bindgen('./pkg/personal_page_bg.wasm');

    if (back_buffer.length !== 0) {
        let data = await load_bytes(location.origin, back_buffer);
        self.postMessage(data);
    }

    self.onmessage = async event => {
        let data = await load_bytes(location.origin, event.data);
        self.postMessage(data);
    }
}

init_worker()
