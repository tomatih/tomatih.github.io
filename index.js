const {main} = wasm_bindgen;

async function run_wasm() {
    await wasm_bindgen();
    main();
}

run_wasm();
