import { init, b64encode, b64decode, LedgerEntry, Issuer, Organizer, Participant, Resource, Participation, ConfirmedParticipation } from 'prepams-shared';

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

        if (ArrayBuffer.isView(out)) {
          self.postMessage([time, out], [out.buffer]);
        } else {
          self.postMessage([time, out]);
        }
      } else {
        console.log('[worker]', job, args);
      }
  }
});

class Evaluation {
  constructor(workload) {
    this.issuer = Issuer.deserialize(b64decode(workload.issuer), []);

    for (const entry of (workload.ledger || [])) {
      this.issuer = this.issuer.appendEntry(LedgerEntry.deserialize(b64decode(entry)));
    }

    this.organizers = new Map(workload.organizers.map(({ id, seed, credential }) => [
      id,
      Organizer.deserialize(b64decode(credential))
    ]));

    this.studies = new Map(workload.studies.map(e => [e.id, {
      reward: e.reward,
      organizer: e.organizer,
      resource: Resource.deserialize(b64decode(e.object))
    }]));

    this.participants = new Map([...workload.PARTICIPANTS, ...workload.WARMUP_PARTICIPANTS]
      .filter(e => e.credential)
      .map(e => [e.id, { credential: Participant.deserialize(b64decode(e.credential)) }])
    );
  }

  clear() {
    this.participants.clear();
  }

  async registerRequest({ id, seed, attributes }) {
    if (this.participants.has(id)) {
      return;
    }

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
    if (!request) {
      return;
    }

    return this.issuer.issueCredential(request);
  }

  async registerComplete({ id }, response) {
    if (!response) {
      return [];
    }

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
    const receipt = this.issuer.checkPayoutRequest(request);
    return receipt.entry.payout.serialize();
  }
};
