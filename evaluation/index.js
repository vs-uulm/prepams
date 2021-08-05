const os = require('os');
const path = require('path');
const express = require('express');
const puppeteer = require('puppeteer');

const { mkdir, writeFile } = require('fs/promises');
const { createArrayCsvWriter } = require('csv-writer');

const cliProgress = require('cli-progress');
const bar = new cliProgress.SingleBar({
  noTTYOutput: true,
  formatValue: (v, options, type) => {
    if (type === 'percentage') {
    return `${String(v).padStart(3)}% | ET: ${new Date(Date.now() - bar.starttimer).toJSON().slice(11, 19).replace(/^(00?:?)*/, '').padStart(8)}s`;
    }
    return String(v).padStart(5);
  }
}, cliProgress.Presets.rect);
bar.ostart = bar.start;
bar.start = (title, task, max) => {
  if (bar.options.title !== title) {
    console.log();
  }
  bar.starttimer = Date.now();
  bar.options.format = `[${title.padStart(10)}] ${String(task).padEnd(35)} ${cliProgress.Presets.rect.format.replace('%', '')}`;
  bar.options.title = title;
  bar.ostart(max, 0);
};

const buildEvalApplication = require('./src/build');
const experiments = require('./experiments');

(async () => {
  for (const config of experiments) {
    try {
      // create results directory
      console.log('[info] writing results to', config.DIR);
      await mkdir(config.DIR, { recursive: true });

      // try to load previously generated workload
      const workload = await config.prepareWorkload(bar);

      // prepare frontend application for evaluation 
      await buildEvalApplication(bar, config);

      // start and prepare eval server
      bar.start('setup', 'start eval server', 1);
      const app = express();
      app.use(express.static(path.join(config.DIR, './dist')));
      const server = await new Promise((resolve) => {
        const s = app.listen(() => resolve(s));
      });
      bar.increment();
      bar.stop();

      // start and prepare eval browser
      bar.start('setup', 'start eval browser', 1);
      const browser = await puppeteer.launch({
        args: ['--no-sandbox', '--disable-setuid-sandbox']
      });
      const page = await browser.newPage();

      page.on('pageerror', ({ message }) => console.log(message));
      page.on('console', async (e) => {
        const args = await Promise.all(e.args().map(a => a.jsonValue()));
        console.error('\n[browser]', ...args);
      });

      await page.goto(`http://127.0.0.1:${server.address().port}`, { waitUntil: 'load' });
      await page.exposeFunction('tick', () => bar.increment());
      await page.waitForFunction('window.init');
      await page.evaluate((workload) => window.init(workload), workload);

      const eval = async (name, workload, jobs) => {
        bar.start('evaluation', name, workload.length);

        const times = await page.evaluate((input, jobs) => window.run(
          input,
          jobs
        ), workload, jobs);

        const columns = Object.keys(times?.[0]);
        const writer = createArrayCsvWriter({
          path: path.join(config.DIR, `${name}.csv`),
          header: columns
        });

        await writer.writeRecords(times.map(row => columns.map(c => row[c]).filter((row, i) => {
          if (!config.filterRecords) {
            return true;
          }

          return config.filterRecord(name, row, i);
        })));
        await writeFile(
          path.join(config.DIR, `${name}.json`),
          JSON.stringify(times, null, 2)
        );
        bar.stop();
      };

      bar.increment();
      bar.stop();

      console.log('[info] OS:', os.type(), os.release());
      console.log('[info] RAM:', (os.totalmem()/1024/1024/1024).toFixed(2), 'GiB');
      console.log('[info] CPU:', os.cpus()[0].model.replace('@', `${os.cpus().length}x`));
      console.log('[info] Browser:', await browser.version());

      // run evaluation jobs
      try {
        if (workload.participants?.length) {
          await eval('register', workload.participants, ['registerRequest', 'registerVerify', 'registerComplete']);
        }
        if (workload.participations?.length) {
          await eval('participations', workload.participations, ['participate', 'reward']);
        }
        if (workload.payouts?.length) {
          await eval('payout', workload.payouts, ['payoutRequest', 'payoutVerify']);
        }
      } catch (e) {
        console.log('[eval error]', config.name, e);
      }

      // stop evaluation processes and cleanup
      bar.start('complete', 'cleanup', 2)
      await browser.close();
      bar.increment();
      server.close();
      bar.increment();
      bar.stop();
      console.log('[info] all done =)');
    } catch(e) {
      console.error('\n\n[fatal error]', config.name, 'failed:\n', e);
    }
  }
})().catch(e => console.error('\n\n[fatal error]', e));
