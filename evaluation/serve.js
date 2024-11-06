const fs = require('fs/promises');
const express = require('express');
const parser = require('ua-parser-js');
const path = require('path');

const app = express();

app.use((req, res, next) => {
  res.header('Cross-Origin-Opener-Policy', 'same-origin');
  res.header('Cross-Origin-Embedder-Policy', 'require-corp');
  res.header('Cache-Control', 'no-store, must-revalidate');
  next();
});

app.use(express.static(path.join(__dirname, 'dist')));

app.post('/post', express.text(), async (req, res) => {
  try {
    const ua = parser(req.headers['user-agent']);
    const deviceType = (ua.device.type === 'mobile' || ua.device.type === 'tablet') ? ua.device.type : 'laptop';

    const prefix = `${ua.os.name}-${ua.os.version}_${ua.browser.name}-${ua.browser.version}`.replace(/[^a-zA-Z0-9-_]*/g, '');
    const experiment = req.query.experiment.replace(/[^a-zA-Z0-9-_]*/g, '');
    const file = `${req.query.file.replace(/[^a-zA-Z0-9-_]*/g, '')}.csv`;

    if (!experiment || !file) {
        return res.status(400).end('Bad Request');
    }

    let dir = path.join(__dirname, 'results', 'browser', `${prefix}_${deviceType}`, experiment);
    if (process.env['OUTPUT_DIR']) {
        dir = path.join(process.env['OUTPUT_DIR'], 'browser', `${prefix}_${deviceType}`, experiment);
    }
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(path.join(dir, file), req.body);
    console.log('wrote', path.join(dir, file));
    res.end('');
  } catch (e) {
    res.end(e.toString());
  }
});

console.log('This evaluation step allows you to evaluate other devices, such as mobile phones and tablets.');
console.log('It will run indefinitely until you manually stop it, by for example hitting [Ctrl] + [c] on the keyboard.');
console.log('An evaluation HTTP server is started on port 52716, open it using your device\'s browser.');

if (process.env['SKIP_STEP']) {
    console.log();
    console.log('Note: Skipping evaluation server step...');
    console.log();
    console.log('To enable evaluation server, remove the following line from .popper.yml:');
    console.log(`   SKIP_STEP: "${process.env['SKIP_STEP']}"}`);
    console.log();
    process.exit(0);
}

app.listen(52716, () => console.log(`Eval server listening on port 52716`));
