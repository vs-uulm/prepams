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

app.use(express.static('dist'));

app.post('/post', express.text(), async (req, res) => {
  const ua = parser(req.headers['user-agent']);
  const prefix = `${ua.os.name}-${ua.os.version}_${ua.browser.name}-${ua.browser.version}`.replace(/[^a-zA-Z0-9-_]*/g, '');
  const experiment = req.query.experiment.replace(/[^a-zA-Z0-9-_]*/g, '');
  const file = `${req.query.file.replace(/[^a-zA-Z0-9-_]*/g, '')}.csv`;

  if (!experiment || !file) {
      return res.status(400).end('Bad Request');
  }

  try {
    const dir = path.join(__dirname, 'results', 'browser', prefix, experiment);
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(path.join(dir, file), req.body);
    console.log('wrote', path.join(dir, file));
    res.end('');
  } catch (e) {
    res.end(e.toString());
  }
});

app.listen(8081, () => console.log(`eval server listening on port 8081`));
