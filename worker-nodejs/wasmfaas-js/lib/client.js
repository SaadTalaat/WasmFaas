const Executor = require("./executor");
const axios = require("axios");
const WebSocket = (() => {
  if (typeof window !== "undefined"){
    return BrowserWebSocketWrapper;
  }
  return require("ws");
})();

/*
 * Expected incoming message schema
{
  type: 'invoke',
  request_id: '2ea49337-4531-4e67-a86e-93f464c8d424',
  name: 'echo3',
  uri: 'assets/echo3_3724494060.wasm',
  signature: {
    params: [],
    shim_idx: 0,
    ret: { type: 'vector', content: [Object] },
    inner_ret: { type: 'vector', content: [Object] }
  },
  args: []
}
*/
class BrowserWebSocketWrapper {
  constructor(url) {
    this.__ws = new WebSocket(url);
  }

  on(evt, callback) {
    switch(evt) {
      case 'open':
        this.__ws.onopen = callback;
        break;
      case 'message':
        this.__ws.onmessage = callback;
        break;
      case 'close':
        this.__ws.onclose = callback;
        break;
      default:
        throw Error("Unsupported event %s", evt);
    }
  }

  close() {
    this.__ws.close();
  }
}

class WasmFaasClient {


  constructor(hostUri, tlsEnabled, kvStore, logger) {

    if (!hostUri || !kvStore || tlsEnabled === undefined) {
      throw Error("(hostUri, tlsEnabled, kvStore) must be provided");
    }
    this.kvStore = kvStore;
    this.logger = logger || (() => {});
    // TODO: Configurable proto
    this.wsUri = (tlsEnabled? "wss" : "ws") +"://" + hostUri + "/ws";
    this.httpBaseUri = (tlsEnabled? "https": "http") + "://" + hostUri + "/";
  }

  onMessage(callback) {
    this.onMessageCallback = callback;
  }

  onClose(callback) {
    this.onCloseCallback = callback;
  }

  async __handleInvoke(msg) {
    let fn = msg.name;
    let uri = msg.uri;
    let signature = msg.signature;
    let args = msg.args;
    var wasmModule = await this.kvStore.getItem(uri);
    if (!wasmModule) {
      let url = this.httpBaseUri + msg.uri;
      let response = await axios.get(
        url,
        {responseType: "arraybuffer"}
      );
      wasmModule = response.data;
      await this.kvStore.setItem(uri, wasmModule);
    }

    return await Executor.execute(wasmModule, fn, signature, args);
  }

  async __processMessage(msg) {
    switch(msg.type) {
      case "invoke":
        {
          this.logger("[WasmFaasClient] Invoke request received, function: %s, id: %s", msg.name, msg.request_id);
          let result = await this.__handleInvoke(msg);
          let reply = {
            type: "result",
            request_id: msg.request_id,
            content: result
          };
          this.logger("[WasmFaasClient] Replying to request: %s", msg.request_id);
          return reply;
        }
      default:
        this.ws.send("Unrecognized request type: %s", msg.type);
        throw Error("Failed to process message");

    }
  }

  start() {
    if (this.__ws)
      throw Error("Client already started");

    let ws = new WebSocket(this.wsUri);
    ws.on('open', async () => {
      this.logger("[WasmFaasClient] WS to %s initiated", this.wsUri);
    })

    ws.on('message', async (data) => {
      if (this.onMessageCallback)
        await this.onMessageCallback(data);

      let msg = JSON.parse(data.toString());
      let result = await this.__processMessage(msg);

      ws.send(JSON.stringify(result));

    })

    ws.on('close', async (data) => {
      if (this.onCloseCallback)
        await this.onCloseCallback(data);
      this.logger("[WasmFaasClient] WS server terminated connection %d", data);
    });

    this.__ws = ws;
  }

  async close() {
    await this.__ws.close();
  }
}

module.exports = WasmFaasClient;
