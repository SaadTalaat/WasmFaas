import React, { useState, useEffect } from "react";
import config from "./config.js";
import WasmFaasClient, {BrowserStore} from "./wasmfaas-js";


const kvStore = new BrowserStore();

const App = () => {
  const [connected, setConnected] = useState(false);
  const [client, setClient] = useState(null);
  const [logs, setLogs] = useState([]);
  const [msg, setMsg] = useState(null);

  const handleClick = async () => {
    if (!connected) {
      const ws = new WasmFaasClient(config.hostUri, config.tlsEnabled, kvStore, setMsg);
      ws.start();
      setClient(ws);
    } else {
      await client.close();
      setClient(null);
    }
    setConnected(!connected);
  };

  useEffect(() => {
    if (msg === null)
      return;
    let now = new Date();
    let datedMsg = "[" + now.toLocaleTimeString() + "] " + msg;
    setLogs([
      ...logs,
      datedMsg
    ]);
    setMsg(null);
  }, [msg])

  return (
    <div style={styles.container}>
      <h3 style={styles.header}>Worker Interface</h3>
      <button
        onClick={handleClick}
        style={connected ? styles.connect : styles.disconnect}
      >
        {connected ? "Disconnect" : "Connect"}
      </button>
      <div style={styles.logs}>
        {
          logs.map((log, index) => {
            return <p style={styles.log} key={index}>{log}</p>;
          })
        }
      </div>
    </div>
  );
};

const styles = {
  container: {
    display: "flex",
    justifyContent: "center",
    flexDirection: "column",
  },
  header: {
    display: "flex",
    alignSelf: "center",
  },
  connect: {
    display: "flex",
    backgroundColor: "green",
    justifyContent: "center",
    width: "10%",
    alignSelf: "center",
  },
  disconnect: {
    display: "flex",
    backgroundColor: "red",
    width: "10%",
    justifyContent: "center",
    alignSelf: "center",
  },
  logs: {
    display: "flex",
    flexDirection: "column",
    marginTop: 10,
    backgroundColor: "rgba(0,0,0)",
    width: "90%",
    height: 300,
    alignSelf: "center",
    border: "1px solid black",
    overflowY: "scroll"
  },
  log: {
    fontSize: 13,
    margin: 4,
    marginLeft: 10,
    color: "rgba(0, 200, 0)"
  },
};

export default App;
