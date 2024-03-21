const os = require('os');
const path = require('path');
const express = require('express');
const puppeteer = require('puppeteer');

const { mkdir, writeFile } = require('fs/promises');
const { createArrayCsvWriter } = require('csv-writer');

const log = require('debug')('prepams:evaluation');

const { buildEvalApplication, buildIndex } = require('./src/build.js');
const progress = require('./src/progress.js');
const experiments = require('./experiments');

(async () => {
  const bar = progress.start('prepare', 'preparing eval', experiments.length + 2);

  // prepare frontend application for evaluation 
  const appPath = path.join(__dirname, 'dist');
  log('[info] writing evaluation application to ', appPath);
  await mkdir(path.join(appPath, 'workloads'), { recursive: true });
  await buildEvalApplication(appPath);
  bar.increment();

  for (const experiment of experiments) {
    bar.updateTitle(experiment.config.NAME);
    try {
      // prepare workload
      await experiment.prepareWorkload();

      bar.increment();
    } catch(e) {
      console.error('\n\n[fatal error]', experiment.config.NAME, 'failed:\n', e);
    }
  }

  bar.updateTitle('creating index');
  await buildIndex(appPath, experiments);
  bar.increment();
  bar.stop();

  if (process.argv[2] === 'evaluate') {
    log('starting eval server', 1);
    const app = express();
    app.use(express.static(appPath));
    const server = await new Promise((resolve) => {
      const s = app.listen(() => resolve(s));
    });

    let summary = progress.start('evaluate', 'running experiments', experiments.length);
    for (const experiment of experiments) {
      let bar = progress.start(experiment.config.NAME, 'starting eval browser', 2 + experiment.workload.length);
      experiment.workload.name = experiment.config.NAME;

      try {
        // create results directory
        log('[info] writing results to', experiment.DIR);
        await mkdir(experiment.DIR, { recursive: true });

        // start and prepare eval browser
        const browser = await puppeteer.launch({
          args: ['--no-sandbox', '--disable-setuid-sandbox'],
          protocolTimeout: 0,
          headless: 'new',
        });
        const page = await browser.newPage();

        page.on('pageerror', ({ message }) => console.error(message));
        page.on('console', async (e) => {
          const args = await Promise.all(e.args().map(a => a.jsonValue()));
          console.error('\n[browser]', ...args);
        });

        await page.goto(`http://127.0.0.1:${server.address().port}/`, { waitUntil: 'load' });

        await page.exposeFunction('tick', () => bar.increment());
        await page.exposeFunction('writeResponses', async (name, times) => {
          try {
            if (times.length) {
              const columns = Object.keys(times?.[0]);
              const writer = createArrayCsvWriter({
                path: path.join(experiment.DIR, `${name}.csv`),
                header: columns
              });
              await writer.writeRecords(
                times
                  .map(row => columns.map(c => row[c]))
                  .filter((row, i) => {
                    if (!experiment.config.filterRecords) {
                      return true;
                    }

                    return experiment.config.filterRecord(name, experiment.config.NAME, row, i);
                  })
              );
              await writeFile(
                path.join(experiment.DIR, `${name}.json`),
                JSON.stringify(times, null, 2)
              );
            }
          } catch (e) {
            console.error('[eval error]', e);
          }
        });
        await page.waitForFunction('window.init');
        await page.evaluate((workload) => window.init(workload, 'headless'), experiment.workload);

        bar.increment();
        bar.updateTask('executing workload');

        const environment = {
          os: `${os.type()} ${os.release}`,
          ram: `${(os.totalmem()/1024/1024/1024).toFixed(2)} GiB`,
          cpu: os.cpus()[0].model.replace('@', `${os.cpus().length}x`),
          browser: await browser.version()
        };

        log('[info] OS:', environment.os);
        log('[info] RAM:', environment.ram);
        log('[info] CPU:', environment.cpu);
        log('[info] Browser:', environment.browser);

        // run evaluation jobs
        try {
          await page.evaluate(() => window.runExperiments());
        } catch (e) {
          console.error('[eval error]', experiment.config.NAME, e);
          break;
        }

        // stop evaluation processes and cleanup
        bar.updateTask('cleanup');
        await browser.close();
        bar.increment();
        log('[info] all done =)');
      } catch(e) {
        console.error('\n\n[fatal error]', experiment.config.NAME, 'failed:\n', e);
      }
      summary.increment();
      bar.stop();
    }
    server.close();
    summary.stop();
  }

  progress.stop();
})().catch(e => console.error('\n\n[fatal error]', e));
