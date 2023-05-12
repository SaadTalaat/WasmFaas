class BrowserStore {
  constructor() {
    this.store = {};
  }

  async getItem(key) {
    let uri = "wasmfaas/" + key;
    return this.store[uri];
  }

  async setItem(key, bytes) {
    let uri = "wasmfaas/" + key;
    return this.store[uri] = bytes;
  }

}

export default BrowserStore;
