const fs = require('fs/promises');

const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
  buildEvalApplication(dir) {
    return new Promise((resolve, reject) => {
      webpack({
        entry: './client/index.js',
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
          path: dir,
          filename: 'index.js',
        },

        plugins: [new HtmlWebpackPlugin()]
      }, (err, stats) => {
        if (err || stats.hasErrors()) {
          return reject(stats?.toJson?.()?.errors?.map?.(e => e.message)?.join?.('\n') || err.message);
        }
        resolve();
      });
    });
  },
  
  async buildIndex(dir, experiments) {
    await fs.writeFile(
      path.join(dir, 'workloads', 'index.json'),
      JSON.stringify(experiments.map(e => e.config.NAME))
    );
  }
};
