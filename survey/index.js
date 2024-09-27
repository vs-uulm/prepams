require('dotenv').config();
const express = require('express');

const { Participation, Organizer, Issuer, ConfirmedParticipation, LedgerEntry, SignedResource } = require('prepams-shared');

const app = express();

const ORGANIZER_ID = process.env['ORGANIZER_ID']
const ORGANIZER_SEED = Buffer.from(process.env['ORGANIZER_SEED'], 'hex');
const ISSUER_PK = Buffer.from(process.env['ISSUER_PK'], 'base64url');
const organizer = new Organizer(ORGANIZER_ID, ISSUER_PK, ORGANIZER_SEED);

app.use(express.static('public'));
app.use(express.json());

const asyncWrapper = fn => (...args) => fn(...args).catch(e => {
  console.log(e);
  args[1].status(e?.status || 500).json({ error: e?.message || 'Internal Server Error' });
});

app.post('/api/submit', asyncWrapper(async (req, res) => {
    const p = Participation.deserialize(Buffer.from(req.body.participation, 'base64'));
    if (!p.verify()) {
        throw new Error('prerequisites not met');
    }

    if (typeof req.body.data?.age !== 'number' || !(req.body.data.age >= 1 && req.body.data.age <= 100)) {
        throw new Error('incomplete submission');
    }

    if (typeof req.body.data?.['BFI-10'] !== 'object' || Object.keys(req.body.data['BFI-10']).length !== 10) {
        throw new Error('incomplete submission');
    }

    const confirmedParticipation = organizer.confirmParticipation(p, req.body.code);
    res.json({ ok: true, confirmation: Buffer.from(confirmedParticipation).toString('base64') });
}));

app.listen(process.env['PORT'], () => console.log(`listening at http://localhost:${process.env['PORT']}`));

