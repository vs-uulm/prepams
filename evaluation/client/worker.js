import { init, b64encode, b64decode, Issuer, Organizer, Participant, Resource, Participation, ConfirmedParticipation } from 'prepams-shared';

setTimeout(() => {
  init();
  self.postMessage('ready');
}, 100);

let evaluation = null;

self.addEventListener('message', async ({ data: [job, args] }) => {
  switch (job) {
    case 'setup':
      evaluation = new Evaluation(...args);
      self.postMessage('done');
      break;
      
    default:
      if (evaluation && evaluation?.[job]) {
        if (job === 'participate') {
          evaluation.updateReferences(...args);
        }

        let out = null;
        const time = {};
        const start = performance.now();
        try {
          out = await evaluation[job](...args, out);
        } catch (e) {
          self.postMessage(JSON.stringify({
            issuer: b64encode(evaluation.issuer.serialize()),
            ledger: b64encode(evaluation.issuer.ledger),
            message: e.toString(),
            args: args.map(e => ArrayBuffer.isView(e) ? b64encode(new Uint8Array(e)) : e),
            job,
            out,
          }, null, 2));

          time[`${job}-error`] = e.toString();
          self.postMessage(e);
          throw e;
        }
        time[job] = performance.now() - start;
        time[`${job}_size`] = out?.length ?? -1;

        self.postMessage([time, out]);
      } else {
        console.log('[worker]', job, args);
      }
  }
});

class Evaluation {
  constructor(workload) {
    this.issuer = Issuer.deserialize(b64decode(workload.issuer), []);

    this.organizers = new Map(workload.organizers.map(({ id, seed, credential }) => [
      id,
      Organizer.deserialize(b64decode(credential))
    ]));

    this.studies = new Map(workload.studies.map(e => [e.id, {
      reward: e.reward,
      organizer: e.organizer,
      resource: Resource.deserialize(b64decode(e.object))
    }]));

    this.participants = new Map();
  }

  clear() {
    this.participants.clear();
  }

  async registerRequest({ id, seed, attributes }) {
    const user = new Participant(
      id,
      new Uint32Array(attributes.map(e => Number(e))),
      this.issuer.ledgerVerificationKey
    );
    this.participants.set(id, {
      credential: user
    });
    return user.requestCredential(
      this.issuer.publicKey,
      this.issuer.verificationKey,
      b64decode(seed)
    );
  }

  async registerVerify(_, request) {
    return this.issuer.issueCredential(request);
  }

  async registerComplete({ id }, response) {
    const user = this.participants.get(id);
    user.credential.retrieveCredential(response);
    return [];
  }

  async updateReferences([i, j, id]) {
    const study = this.studies.get(j);
    study.resource.updateReferences(this.issuer);
    return [];
  }

  async participate([i, j, id]) {
    const user = this.participants.get(i);
    const study = this.studies.get(j);
    return user.credential.participate(study.resource);
  }

  async confirm([i, j, id], data) {
    const study = this.studies.get(j);
    const participation = Participation.deserialize(data);
    if (!participation.verify()) {
      throw new Error('invalid participation');
    }

    const organizer = this.organizers.get(study.organizer);
    return organizer.confirmParticipation(participation, id);
  }

  async reward([i, j, id], data) {
    const study = this.studies.get(j);
    const organizer = this.organizers.get(study.organizer);

    const confirmedParticipation = ConfirmedParticipation.deserialize(data);
    const reward = this.issuer.issueReward(confirmedParticipation, organizer.publicKey, study.reward);
    return reward.serialize();
  }

  async paddingRequest([id, value]) {
    const user = this.participants.get(id);
    user.nullRequest = user.credential.requestNulls();
    return user.nullRequest.request();
  }

  async paddingResponse([id, value], request) {
    return this.issuer.issueNulls(request);
  }

  async payoutRequest([id, value], nullResponse) {
    const user = this.participants.get(id);
    const nulls = user.nullRequest.unblind(nullResponse);
    delete user.nullRequest;

    user.nulls = nulls;
    /*
    console.log('--------------');
    console.log('payout', value, id);
    console.log('ledger', b64encode(this.issuer.ledger));
    console.log('cred', b64encode(user.credential.serialize()));
    console.log('nulls', b64encode(nulls));
    console.log('--------------');
    */

    const request = user.credential.requestPayout(
      value,
      'target',
      id,
      nulls,
      this.issuer.ledger
    );

    return request.proof;
  }

  async payoutVerify([id, value], request) {
    try {
      const receipt = this.issuer.checkPayoutRequest(request);
      return receipt.entry.payout.serialize();
    } catch (e) {
      const user = this.participants.get(id);
      console.log('--------------');
      console.log('value', value);
      console.log('cred', b64encode(user.credential.serialize()));
      console.log('nulls', b64encode(user.nulls));
      console.log('request', b64encode(request));
      console.log('issuer', b64encode(this.issuer.serialize()));
      console.log('ledger', b64encode(this.issuer.ledger));
      console.log('--------------');
      throw e;
    }
  }
};
