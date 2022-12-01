const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = function buildEvalApplication(bar, config) {
  bar.start('setup', 'build eval application', 1);
  return new Promise((resolve, reject) => {
    webpack({
      entry: './evaluation.js',
      mode: 'production',

      target: 'web',

      devtool: 'source-map',
      externals: ['worker_threads', 'ws', 'perf_hooks'],

      resolve: {
        symlinks: true,
        modules: [ 'node_modules' ],
      },

      experiments: {
        asyncWebAssembly: true
      },

      output: {
        path: path.join(config.DIR, './dist/'),
        filename: 'index.js',
      },

      plugins: [new HtmlWebpackPlugin()]
    }, (err, stats) => {
      if (err || stats.hasErrors()) {
        return reject(stats?.toJson?.()?.errors?.map?.(e => e.message)?.join?.('\n') || err.message);
      }
      bar.increment();
      bar.stop();
      resolve();
    });
  });
};
