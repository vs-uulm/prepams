require('dotenv').config();
const uuid = require('uuid');
const helmet = require('helmet');
const morgan = require('morgan');
const express = require('express');
const { Issuer } = require('prepams-shared');
const { body, validationResult } = require('express-validator');

const { openDatabase } = require('./src/utils/db');
const { BadRequest } = require('./src/utils/errors');
const { Reward } = require('prepams-shared');

const app = express();

app.use(helmet());
app.use(express.json());
app.use(morgan('dev'));

const asyncWrapper = fn => (...args) => fn(...args).catch(e => args[1].status(e?.status || 500).json({ error: e?.message || 'Internal Server Error' }));

const validate = (validations) => async (req, res, next) => {
  await Promise.all(validations.map(validation => validation.run(req)));

  const errors = validationResult(req);
  if (errors.isEmpty()) {
    return next();
  }

  return res.status(400).json({ errors: errors.array() });
};

if (process.argv[2] === '--init') {
  console.log('Creating new issuer keys...');
  const issuer = new Issuer();
  console.log('Add the following line to your .env file:');
  console.log();
  console.log(`ISSUER_SECRET="${issuer.serializeBase64()}"`)
  console.log();
  process.exit(0);
}

let issuer;
try {
  issuer = Issuer.deserializeBase64(process.env['ISSUER_SECRET']);
} catch (e) {
  console.error('Error: Issuer credential missing, initialize issuer using --init argument');
  process.exit(1);
}

(async () => {
  const db = await openDatabase();

  app.get('/api/auth/signup', (req, res) => {
    res.json({ publicKey: issuer.publicKey });
  });

  app.get('/api/auth/signin', asyncWrapper(async (req, res) => {
    if (req.query.role === 'participant') {
      const rows = await db.all('SELECT signature FROM issued');
      res.json({
        issuer: { publicKey: issuer.publicKey },
        log: rows.map(e => JSON.parse(e.signature))
      });
    } else {
      const rows = await db.all('SELECT publicKey FROM users WHERE publicKey IS NOT NULL');
      res.json({
        issuer: { publicKey: issuer.publicKey },
        publicKeys: rows.map(e => e.publicKey)
      });
    }
  }));

  app.post('/api/auth/signup', asyncWrapper(async (req, res) => {
    if (await db.get('SELECT * FROM users WHERE id = ?', req.body.id)) {
      // throw new Error('id already registered');
    }

    if (req.query.role === 'participant') {
      const signature = issuer.issueCredential(req.body);
      await db.run('INSERT INTO users (id, role) VALUES (?, ?)', req.body.id, 'participant');
      await db.run('INSERT INTO issued (signature) VALUES (?)', JSON.stringify(signature));
      res.json(signature);
    } else {
      await db.run('INSERT INTO users (id, role, publicKey) VALUES (?, ?, ?)', req.body.id, req.query.role, req.body.publicKey);
      res.json({ ok: true });
    }
  }));

  app.get('/api/studies', asyncWrapper(async (req, res) => {
    const rows = req.query.id ? await db.all('SELECT * FROM studies WHERE owner = ?', req.query.id) : await db.all('SELECT * FROM studies');

    res.json(rows.map((row) => {
      row.qualifier = JSON.parse(row.qualifier);
      row.disqualifier = JSON.parse(row.disqualifier);
      return row;
    }));
  }));

  app.post('/api/studies', validate([
    body('name').not().isEmpty().trim().escape(),
    body('id').isBase64({ urlSafe: true }).isLength({ min: 43, max: 43 }),
    body('owner').not().isEmpty().trim().escape(),
    body('abstract').not().isEmpty().trim().escape(),
    body('description').not().isEmpty().trim().escape(),
    body('duration').not().isEmpty().trim().escape(),
    body('reward').isInt({ min: 0 }).toInt(),
    body('qualifier').isArray(),
    body('qualifier.*').isBase64({ urlSafe: true }).isLength({ min: 43, max: 43 }),
    body('disqualifier').isArray(),
    body('disqualifier.*').isBase64({ urlSafe: true }).isLength({ min: 43, max: 43 }),
    body('webBased').isBoolean().toBoolean(),
    body('studyURL').optional({ checkFalsy: true }).isURL().trim(),
    body('signature').isBase64({ urlSafe: true }).isLength({ min: 86, max: 86 })
  ]), asyncWrapper(async (req, res) => {
    const row = await db.get('SELECT publicKey FROM users WHERE id = ?', req.body.owner);

    const resource = [
      req.body.owner,
      req.body.id,
      req.body.name,
      req.body.abstract,
      req.body.description,
      req.body.duration,
      req.body.reward,
      req.body.qualifier.sort().join(','),
      req.body.disqualifier.sort().join(','),
      req.body.webBased,
      req.body.studyUrl,
    ].join('|');

    if (!issuer.checkResourceSignature(resource, req.body.signature, row.publicKey)) {
      throw new BadRequest('signature not valid');
    }

    req.body.qualifier = JSON.stringify(req.body.qualifier);
    req.body.disqualifier = JSON.stringify(req.body.disqualifier);

    await db.run(`
      INSERT INTO studies (
        id,
        name,
        owner,
        abstract,
        description,
        duration,
        reward,
        qualifier,
        disqualifier,
        webBased,
        studyURL
      ) VALUES (
        :id,
        :name,
        :owner,
        :abstract,
        :description,
        :duration,
        :reward,
        :qualifier,
        :disqualifier,
        :webBased,
        :studyURL
      )
    `, {
      ':id': req.body.id,
      ':name': req.body.name,
      ':owner': req.body.owner,
      ':abstract': req.body.abstract,
      ':description': req.body.description,
      ':duration': req.body.duration,
      ':reward': req.body.reward,
      ':qualifier': req.body.qualifier,
      ':disqualifier': req.body.disqualifier,
      ':webBased': req.body.webBased,
      ':studyURL': req.body.studyUrl
    });

    res.status(201).json({ ok: true, id: req.body.id });
  }));

  app.get('/api/participations', asyncWrapper(async (req, res) => {
    const rows = await db.all('SELECT * FROM participations');
    res.json(rows);
  }));

  app.post('/api/participations', validate([
    body('data').not().isEmpty().isHexadecimal(),
    body('key').isBase64({ urlSafe: true }).isLength({ min: 64, max: 64 }),
    body('iv').not().isEmpty().isHexadecimal().isLength({ min: 24, max: 24 }),
  ]), asyncWrapper(async (req, res) => {
    const participationId = uuid.v4();
    await db.run(`INSERT INTO participations (id, iv, pk, data) VALUES (:id, :iv, :pk, :data)`, {
      ':id': participationId,
      ':iv': req.body.iv,
      ':pk': req.body.pk,
      ':data': req.body.data,
    });

    res.status(201).json({
      ok: true,
      id: participationId,
      url: `${process.env['APP_URL']}/participation/${participationId}`
    });
  }));

  app.get('/api/participations/:id', asyncWrapper(async (req, res) => {
    const participation = await db.get('SELECT id, iv, data FROM participations WHERE id = ?', req.params.id);
    if (!participation) {
      return res.status(404).end('Not Found');
    }

    return res.json(participation);
  }));

  app.get('/api/rewards/:id?', asyncWrapper(async (req, res) => {
    const [transactions, spend] = await Promise.all([
        db.all('SELECT value, id, pk, tag FROM rewards WHERE id = :id OR :id IS NULL', req.params.id),
        db.all('SELECT * FROM spend')
    ]);

    res.json({ transactions, spend });
  }));

  app.post('/api/rewards', asyncWrapper(async (req, res) => {
    const rows = await db.all('SELECT publicKey FROM users WHERE publicKey IS NOT NULL');
    const approvedKeys = rows.map(e => e.publicKey);
    const reward = Reward.deserialize(req.body);
    if (!Issuer.checkRewardSignature(reward, approvedKeys)) {
      throw new BadRequest('reward signature invalid');
    }

    const study = await db.get('SELECT reward FROM studies WHERE id = ?', req.body.id);

    if (study.reward !== req.body.value) {
      throw new BadRequest('issued reward does not match study reward');  
    }

    const data = Buffer.concat([
      Buffer.from(req.body.signature.R_bytes),
      Buffer.from(req.body.signature.s_bytes)
    ]).toString('base64url');

    const issued = await db.get('SELECT 1 FROM rewards WHERE pk = ?', req.body.key);
    if (issued) {
      throw new BadRequest('reward for this participation already issued');
    }

    await db.run('INSERT INTO rewards (pk, id, tag, data, value) VALUES (:pk, :id, :tag, :data, :value); DELETE FROM participations WHERE pk = :pk', {
      ':pk': req.body.key,
      ':id': req.body.id,
      ':tag': req.body.tag,
      ':value': req.body.value,
      ':data': data
    });

    res.json({ ok: true });
  }));

  app.post('/api/payout', asyncWrapper(async (req, res) => {
    const [transactions, spend] = await Promise.all([
        db.all('SELECT value, id, pk FROM rewards'),
        db.all('SELECT tag FROM spend')
    ]);
    const receipt = issuer.checkPayoutRequest(req.body, transactions, spend.map(e => e.tag));
    await Promise.all(req.body.inputs.tags.map(tag => db.run('INSERT INTO spend VALUES (?)', tag)));
    await db.run(`INSERT INTO payouts (recipient, value, target, receipt) VALUES (:recipient, :value, :target, :receipt)`, {
      ':recipient': req.body.inputs.recipient,
      ':target': req.body.inputs.target,
      ':value': req.body.inputs.value,
      ':receipt': receipt
    });
    res.json({ ok: true, receipt });
  }));

  app.listen(process.env['PORT'], () => console.log(`listening at http://localhost:${process.env['PORT']}`));
})();
