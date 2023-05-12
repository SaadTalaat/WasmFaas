const path = require("path");
const fs = require("fs").promises;

class FileSystemStore {
  baseDirectory = "faas_assets";
  constructor() {
    const fsSync = require("fs");
    let baseDirExists = fsSync.existsSync(this.baseDirectory);
    if(!baseDirExists) {
      fsSync.mkdirSync(this.baseDirectory);
    }
  }

  async getItem(key) {
    let filename = path.basename(key);
    let uri = path.join(this.baseDirectory, filename);
    try {
      return await fs.readFile(uri);
    } catch(e) {
      return null;
    }
  }

  async setItem(key, bytes) {
    let filename = path.basename(key);
    let uri = path.join(this.baseDirectory, filename);
    return await fs.writeFile(uri, bytes);
  }
}

module.exports.FileSystemStore = FileSystemStore;

