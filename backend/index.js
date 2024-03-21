require('dotenv').config();
const uuid = require('uuid');
const helmet = require('helmet');
const morgan = require('morgan');
const express = require('express');

const { Issuer, ConfirmedParticipation, LedgerEntry, SignedResource } = require('prepams-shared');

const { openDatabase } = require('./src/utils/db');
const { BadRequest } = require('./src/utils/errors');

const app = express();

app.use(helmet());
app.use(express.json());
app.use(express.raw());
app.use(morgan('dev'));
app.use((req, res, next) => {
  res.sendBinary = (data, status=200) => {
    res.header('Content-Type', 'application/octet-stream');
    res.status(status).send(Buffer.from(data));
  };
  next();
})

const asyncWrapper = fn => (...args) => fn(...args).catch(e => {
  console.log(e);
  args[1].status(e?.status || 500).json({ error: e?.message || 'Internal Server Error' });
});

const ATTRIBUTES = JSON.parse(process.env.ATTRIBUTES);

if (process.argv[2] === '--init') {
  console.log('Creating new issuer keys...');
  console.log(ATTRIBUTES.length, 'attributes: ', ATTRIBUTES.map(e => e[0]).join(', '));
  const issuer = new Issuer(ATTRIBUTES.length, []);
  console.log('Add the following line to your .env file:');
  console.log();
  console.log(`ISSUER_SECRET="${Buffer.from(issuer.serialize()).toString('base64url')}"`);
  console.log();
  process.exit(0);
}

(async () => {
  const db = await openDatabase();

  let issuer;
  try {
    const entries = await db.all('SELECT * FROM ledger ORDER BY id ASC');
    issuer = entries.reduce((issuer, entry) => {
      if (entry.participation) {
        const participation = ConfirmedParticipation.from(entry.participation, entry.tag, entry.study, entry.request, entry.signature, entry.value);
        return issuer.appendEntry(LedgerEntry.fromTransaction(issuer.head, participation, entry.coin, entry.chain));
      } else {
        return issuer.appendEntry(LedgerEntry.fromPayout(issuer.head, entry.coin, entry.chain));
      }
    }, Issuer.deserialize(Buffer.from(process.env['ISSUER_SECRET'], 'base64url'), []));
  } catch (e) {
    console.error('Error: Issuer credential missing, initialize issuer using --init argument');
    console.error(e);
    process.exit(1);
  }

  app.get('/api/issuer/attributes', (req, res) => res.json(ATTRIBUTES));
  app.get('/api/issuer/pk', (req, res) => res.sendBinary(issuer.publicKey));
  app.get('/api/issuer/vk', (req, res) => res.sendBinary(issuer.verificationKey));
  app.get('/api/ledger/vk', (req, res) => res.sendBinary(issuer.ledgerVerificationKey));
  app.get('/api/ledger', (req, res) => res.sendBinary(issuer.ledger));
  app.post('/api/nulls', (req, res) => res.sendBinary(issuer.issueNulls(req.body)));

  app.post('/api/auth/signup', asyncWrapper(async (req, res) => {
    if (await db.get('SELECT * FROM users WHERE id = ?', req.query.id)) {
      // throw new Error('id already registered');
    }

    if (req.query.role === 'participant') {
      const signature = issuer.issueCredential(req.body);
      await db.run('INSERT INTO users (id, role) VALUES (?, ?)', req.query.id, 'participant');
      await db.run('INSERT INTO issued (signature) VALUES (?)', signature);
      res.sendBinary(signature);
    } else {
      await db.run('INSERT INTO users (id, role, publicKey) VALUES (?, ?, ?)', req.query.id, req.query.role, req.body);
      res.json({ ok: true });
    }
  }));

  app.get('/api/auth/signin', asyncWrapper(async (req, res) => {
    const response = {
      issuer: {
        pk: Buffer.from(issuer.publicKey).toString('base64'),
        vk: Buffer.from(issuer.verificationKey).toString('base64')
      },
      ledger: {
        vk: Buffer.from(issuer.ledgerVerificationKey).toString('base64')
      }
    };

    if (req.query.role === 'participant') {
      const rows = await db.all('SELECT signature FROM issued');
      response.log = rows.map(e => e.signature.toString('base64'));
    } else {
      const rows = await db.all('SELECT publicKey FROM users WHERE publicKey IS NOT NULL');
      response.publicKeys = rows.map(e => e.publicKey.toString('base64'));
    }

    res.json(response);
  }));

  app.get('/api/studies', asyncWrapper(async (req, res) => {
    const rows = req.query.id
      ? await db.all('SELECT * FROM studies WHERE owner = ?', req.query.id)
      : await db.all('SELECT * FROM studies');

    res.json(rows.map((row) => {
      row.qualifier = JSON.parse(row.qualifier);
      row.disqualifier = JSON.parse(row.disqualifier);
      row.constraints = JSON.parse(row.constraints);

      return row;
    }));
  }));

  app.post('/api/studies', asyncWrapper(async (req, res) => {
    try {
    let signedResource = SignedResource.deserialize(req.body);

    const row = await db.get('SELECT publicKey FROM users WHERE id = ?', signedResource.owner);

    if (!issuer.checkResourceSignature(signedResource, row.publicKey)) {
      throw new BadRequest('signature not valid');
    }

    const resource = signedResource.resource;

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
        attributes,
        webBased,
        studyURL,
        signature
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
        :attributes,
        :webBased,
        :studyURL,
        :signature
      )
    `, {
      ':id': resource.id,
      ':name': resource.name,
      ':owner': signedResource.owner,
      ':abstract': resource.summary,
      ':description': resource.description,
      ':duration': resource.duration,
      ':reward': resource.reward,
      ':qualifier': JSON.stringify(resource.qualifier),
      ':disqualifier': JSON.stringify(resource.disqualifier),
      ':attributes': JSON.stringify(resource.attributes),
      ':webBased': resource.webBased,
      ':studyURL': resource.studyUrl,
      ':signature': signedResource.signature
    });

    res.status(201).json({ ok: true, id: resource.id });
    } catch (e) {
      console.log(e);
    }
  }));

  app.get('/api/participations', asyncWrapper(async (req, res) => {
    const rows = await db.all('SELECT * FROM participations');
    res.json(rows);
  }));

  app.post('/api/participations', asyncWrapper(async (req, res) => {
    const iv = req.body.slice(0, 12);
    const data = req.body.slice(12);

    const participationId = uuid.v4();
    await db.run(`INSERT INTO participations (id, iv, data) VALUES (:id, :iv, :data)`, {
      ':id': participationId,
      ':iv': iv,
      ':data': data,
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

    return res.sendBinary(Buffer.concat([participation.iv, participation.data]));
  }));

  app.get('/api/rewards/:id?', asyncWrapper(async (req, res) => {
    const transactions = await db.all('SELECT value, study, tag, coin FROM ledger WHERE study = :id OR :id IS NULL', req.params.id);
    res.json({ transactions });
  }));

  app.post('/api/rewards', asyncWrapper(async (req, res) => {
    const participation = ConfirmedParticipation.deserialize(req.body);

    const { id, tag, study, request, signature, value } = participation;

    const { reward, publicKey } = await db.get(`
      SELECT reward, publicKey
      FROM users
        JOIN studies ON studies.owner = users.id
      WHERE publicKey IS NOT NULL AND studies.id = ?
    `, [ study ]);

    const entry = issuer.issueReward(participation, publicKey, reward);

    const issued = await db.get('SELECT 1 FROM ledger WHERE tag = ?', tag);
    if (issued) {
      throw new BadRequest('reward for this participation already issued');
    }

    const row = await db.get('SELECT iv, data FROM participations WHERE id = ?', id);
    if (!row) {
      throw new BadRequest('participation does not exist');
    }

    await db.run(`
      INSERT INTO ledger (participation, tag, iv, data, study, request, signature, value, coin, chain)
        VALUES (:participation, :tag, :iv, :data, :study, :request, :signature, :value, :coin, :chain);
      DELETE FROM participations WHERE id = :participation
    `, {
      ':participation': id,
      ':tag': tag,
      ':iv': row.iv,
      ':data': row.data,
      ':study': study,
      ':request': request,
      ':signature': signature,
      ':value': value,
      ':coin': entry.transaction.coin,
      ':chain': entry.signature,
    });

    res.sendBinary(entry.serialize());
  }));

  app.post('/api/payout', asyncWrapper(async (req, res) => {
    const payout = issuer.checkPayoutRequest(req.body);
    const entry = payout.entry;
    const receipt = entry.payout.serialize();

    await db.run(`
      INSERT INTO ledger (participation, tag, iv, data, study, request, signature, value, coin, chain)
        VALUES (NULL, :tag, NULL, NULL, NULL, NULL, NULL, :value, :coin, :chain)
    `, {
      ':tag': JSON.stringify({ target: payout.target, recipient: payout.recipient }),
      ':value': entry.payout.value,
      ':coin': receipt,
      ':chain': entry.signature
    });

    res.json({ receipt: Buffer.from(receipt).toString('base64') });
  }));

  const { populateDemoData, demoIdentities } = require('./src/utils/demo');

  await populateDemoData(db, issuer, ATTRIBUTES);
  app.get('/api/demo/credentials', (req, res) => res.json(demoIdentities));

  app.get('/api/demo/payouts', asyncWrapper(async (req, res) => {
    const rows = await db.all('SELECT * FROM ledger WHERE "participation" IS NULL');
    const payouts = [];

    for (const row of rows) {
      try {
        row.tag = JSON.parse(row.tag);
        payouts.push({
          id: row.id,
          recipient: row.tag.recipient,
          target: row.tag.target,
          value: row.value,
          receipt: row.coin.toString('base64')
        });
      } catch {
        // ignore
      }
    }

    res.json(payouts);
  }));

  app.listen(process.env['PORT'], () => console.log(`listening at http://localhost:${process.env['PORT']}`));
})();
