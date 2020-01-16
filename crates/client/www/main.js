import init, { run_app } from '../pkg/zn_client.js';
async function main() {
    await init('http://localhost:8085/pkg/zn_client_bg.wasm');
    run_app();
}
main()