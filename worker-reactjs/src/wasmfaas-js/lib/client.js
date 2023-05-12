import Executor from "./executor.js";
import axios from "axios";
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

  send(msg) {
    this.__ws.send(msg)
  }

  close() {
    this.__ws.close();
  }
}

class WasmFaasClient {


  constructor(hostUri,  tlsEnabled, kvStore, logger) {

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
      wasmModule = new Uint8Array(response.data);
      await this.kvStore.setItem(uri, wasmModule);
    }

    return await Executor(wasmModule, fn, signature, args);
  }

  async __processMessage(msg) {
    switch(msg.type) {
      case "invoke":
        {
          this.logger(`[WasmFaasClient] Invoke request received, function: ${msg.name}, id: ${msg.request_id}`);
          let result = await this.__handleInvoke(msg);
          let reply = {
            type: "result",
            request_id: msg.request_id,
            content: result
          };
          this.logger(`[WasmFaasClient] Replying to request: ${msg.request_id}`);
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

    let ws = new BrowserWebSocketWrapper(this.wsUri);
    ws.on('open', async () => {
      this.logger(`[WasmFaasClient] WS to ${this.wsUri} initiated`);
    })

    ws.on('message', async (wsmsg) => {
      if (this.onMessageCallback)
        await this.onMessageCallback(wsmsg);

      let msg = JSON.parse(wsmsg.data.toString());
      let result = await this.__processMessage(msg);

      ws.send(JSON.stringify(result));

    })

    ws.on('close', async (data) => {
      if (this.onCloseCallback)
        await this.onCloseCallback(data);
      this.logger(`[WasmFaasClient] WS server terminated connection ${data}`);
    });

    this.__ws = ws;
  }

  async close() {
    await this.__ws.close();
  }
}

export default WasmFaasClient;
