module.exports = {
  devServer: {
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
    experiments: {
      asyncWebAssembly: true
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
    // 'vuex-persist',
    'vuetify'
  ]
}
