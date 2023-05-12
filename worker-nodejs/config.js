let env = process.env.FAAS_ENV;

let config = {
  "dev": {
    "host": "api",
    "port": "8090",
    "tlsEnabled": false,
  },
  "staging": {
    "host": "api",
    "port": "8090",
    "tlsEnabled": false,
  }
};

module.exports = config[env];
