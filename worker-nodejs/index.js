const {WasmFaasClient, FileSystemStore} = require("./wasmfaas-js");
const {hostname, port, tlsEnabled} = require("./config");


let kvStore = new FileSystemStore();
let client = new WasmFaasClient("0.0.0.0", 8090, false, kvStore, console.log);
client.start();
