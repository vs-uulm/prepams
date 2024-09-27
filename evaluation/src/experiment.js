const { createHash } = require('crypto');
const fs = require('fs/promises');
const path = require('path');

const { Random, MersenneTwister19937 } = require('random-js');
const Prob = require('prob.js');

const progress = require('./progress.js');

const log = require('debug')('prepams:experiment');

module.exports = class Experiment {
  constructor(config) {
    this.config = config;

    const hash = createHash('sha256');
    hash.update(JSON.stringify(
      Object.entries(this.config)
        .sort((a, b) => a[0].localeCompare(b[0]))
        .filter(([k, v]) => typeof v !== 'function')
        .map(([k, v]) => [k, JSON.stringify(v)])
    ));

    this.digest = hash.digest('base64');
    this.WORKLOAD = path.normalize(path.join(__dirname, '..', 'dist', 'workloads', `${this.config.NAME}.json`));

    this.DIR = path.normalize(path.join(__dirname, '..', 'results', this.config.NAME));
    if (process.env['OUTPUT_DIR']) {
        this.DIR = path.normalize(path.join(process.env['OUTPUT_DIR'], this.config.NAME));
    }

    const seed = [...new Uint32Array(new Uint8Array(Buffer.from(this.config.SEED, 'hex')).buffer)];
    this.prng = MersenneTwister19937.seedWithArray(seed);
    const next = () => this.prng.next();

    for (const [key, params] of Object.entries(config.DISTRIBUTIONS)) {
      const distribution = Prob[params[0]](...params.slice(1));
      this[`${key}Distribution`] = () => distribution(next);
    }
  }

  generateWorkload() {
    if (typeof(this.config.generateWorkload) !== 'function') {
      throw new Error('generateWorkload has to be provided by experiment');
    }
    return this.config.generateWorkload.call(this, progress, this.config, new Random(this.prng));
  }

  async prepareWorkload() {
    let workload;
    try {
      const buf = await fs.readFile(this.WORKLOAD, 'utf-8');
      workload = JSON.parse(buf);
      if (workload.digest !== this.digest) {
        log('[info] previously generated workload is out of date, regenerating...');
        throw new Error();
      }
      log('[info] loaded previously generated workload');
    } catch (e) {
      log('[info] generate workload');
      try {
        workload = await this.generateWorkload();
        workload.digest = this.digest;
        workload.filterRecord = this.config.filterRecord || 'true';
        workload.filterWorkload = (this.config.filterWorkload).toString();
        await fs.writeFile(this.WORKLOAD, JSON.stringify(workload));
      } catch (e) {
        console.error(e);
        process.exit(1);
      }
    }
    this.workload = workload;
    return workload;
  }

  getWorkload() {
    if (typeof(this.config.filterWorkload) !== 'function') {
      return this.workload;
    }

    return this.config.filterWorkload.call(this, this.workload, process.env['WORKLOAD_SIZE']);
  }
};
