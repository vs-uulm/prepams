module.exports = {
  apps : [{
    name: 'prepams-auth-provider',
    cwd: './auth-provider/',
    script: 'index.js'
  }, {
    name: 'prepams-backend',
    cwd: './backend/',
    script: 'index.js'
  }, {
    name: 'prepams-payment-network',
    exec_interpreter: 'none',
    script: 'monerod',
    args: '--testnet --no-zmq --no-igd --hide-my-port --offline --fixed-difficulty 2 --log-level 0'
  }]
}
