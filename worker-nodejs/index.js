const {WasmFaasClient, FileSystemStore} = require("./wasmfaas-js");
const config = require("./config");


let kvStore = new FileSystemStore();
console.log(config);
let client = new WasmFaasClient(config.hostUri, config.tlsEnabled, kvStore, console.log);
client.start();
