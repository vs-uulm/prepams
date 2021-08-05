import { Issuer, Organizer, Participant, Resource, Participation } from 'prepams-shared';

const fromHex = s => new Uint8Array(s.match(/.{1,2}/g).map(b => parseInt(b, 16)));
const encoder = new TextEncoder();

async function deriveKeys(id, password) {
  const keyMaterial = await window.crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    { name: 'PBKDF2' },
    false,
    ['deriveBits', 'deriveKey']
  );

  const salt = await crypto.subtle.digest('SHA-256', encoder.encode(id));
  const decryptionKey = await window.crypto.subtle.deriveKey({
    name: 'PBKDF2',
    salt: salt,
    iterations: 100000,
    hash: 'SHA-256'
  }, keyMaterial, {
    name: 'AES-GCM',
    length: 256
  }, true, [ 'encrypt', 'decrypt' ]);

  const seed = await window.crypto.subtle.deriveBits({
    name: 'PBKDF2',
    salt: new Uint8Array(salt).reverse(),
    iterations: 100000,
    hash: 'SHA-256'
  }, keyMaterial, 256);

  return new Uint8Array(seed);
}

class Evaluation {
  constructor(workload) {
    this.issuer = Issuer.deserializeBase64(workload.issuer);

    this.organizers = new Map(workload.organizers.map(({ id, seed }) => [
      id,
      new Organizer(id, this.issuer.publicKey, fromHex(seed))
    ]));

    this.studies = new Map(workload.studies.map(e => [e.id, e]));

    this.participants = new Map();

    this.database = {
      transactions: [],
      spend: []
    };
  }

  async registerRequest(id) {
    const seed = await deriveKeys(id, id);
    const user = new Participant(id);
    this.participants.set(id, user);
    return user.requestCredential(this.issuer.publicKey, seed);
  }

  async registerVerify(_, request) {
    return this.issuer.issueCredential(request);
  }

  async registerComplete(id, response) {
    const user = this.participants.get(id);
    user.retrieveCredential(response);
    return {};
  }

  async participate([i, j]) {
    const participant = this.participants.get(i);
    const study = this.studies.get(j);

    const resource = Resource.deserialize({
      id: study.id,
      reward: study.reward,
      qualifier: study.qualifier.map(id => ({
        id,
        tags: this.database.transactions.filter(e => e.id === id).map(e => e.tag)
      })),
      disqualifier: study.disqualifier.map(id => ({
        id,
        tags: this.database.transactions.filter(e => e.id === id).map(e => e.tag)
      }))
    });

    const participation = participant.participate(resource);
    return participation.serializeBinary();
  }

  async reward([i, j], data) {
    const p = Participation.deserializeBinary(new Uint8Array(data));
    const study = this.studies.get(j);
    const organizer = this.organizers.get(study.organizer);
    if (!organizer.checkParticipation(p)) {
      throw new Error('invalid participation');
    }

    const { prerequisitesProof } = p.serialize();
    const reward = organizer.issueReward(p).serialize();
    reward.pk = reward.key;
    this.database.transactions.push(reward);

    return {
      qualifier: study.numQualifiers,
      qualifiers: prerequisitesProof.inputs.qualifiers.reduce((a, q) => a + q.tags.length, 0),
      disqualifier: study.numDisqualifiers,
      disqualifiers: prerequisitesProof.inputs.disqualifiers.reduce((a, q) => a + q.tags.length, 0),
      transactions: this.database.transactions.length
    };
  }

  async payoutRequest([id, value]) {
    const participant = this.participants.get(id);
    const req = participant.requestPayout(
      value,
      'target',
      id,
      this.database.transactions,
      this.database.spend
    );
    return req[1];
  }

  async payoutVerify(_, request) {
    this.issuer.checkPayoutRequest(
      request,
      this.database.transactions,
      this.database.spend
    );

    this.database.spend.push(...request.inputs.tags);

    return {
      inputs: request.inputs.inputs,
      set: request.inputs.transactions.length,
      transactions: this.database.transactions.length
    };
  }
};

window.init = function(...args) {
  window.evaluation = new Evaluation(...args);
};

window.run = async function(input, jobs) {
  const times = [];
  for (const args of input) {
    let out = null;
    const time = {};

    for (const job of jobs) {
      const start = performance.now();
      try {
        out = await window.evaluation[job](args, out);
      } catch (e) {
        time[`${job}-error`] = e.toString();
      }
      time[job] = performance.now() - start;
      time[`${job}_size`] = JSON.stringify(out).length;
    }

    Object.assign(time, out);
    await window.tick();
    times.push(time);
  }
  return times;
};
