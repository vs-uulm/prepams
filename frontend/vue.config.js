module.exports = {
  devServer: {
    port: 8008,
    allowedHosts: 'all',
    client: {
      webSocketURL: 'auto://0.0.0.0:0/ws'
    },
    proxy: {
      '/api': {
        target: 'http://localhost:3008'
      }
    }
  },

  configureWebpack: {
    target: 'web',
    experiments: {
      asyncWebAssembly: true
    },
    performance: {
      hints: false
    },
    plugins: [{
      apply(compiler) {
        compiler?.hooks?.done?.tap?.('prepams-shared', (stats) => {
          if (stats.compilation.errors?.[0]?.message?.includes?.('prepams-shared')) {
            process.exit(1);
          }
        });
      }
    }]
  },

  transpileDependencies: [
    'vuetify'
  ],
}
