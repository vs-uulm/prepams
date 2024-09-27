const { init, Issuer, Participant, Organizer, Resource } = require('prepams-shared');
const { createHash } = require('crypto');
const { faker } = require('@faker-js/faker');

const Experiment = require('../src/experiment');
init();

// workload used in PETS'25 evaluation
const SET_SIZE = 128;
const SET_SAMPLE_SIZE = 32;
const WARMUP_PARTICIPANTS = 50;
const PARTICIPANTS = 100;

module.exports = ['qualifier', 'disqualifier', 'set-constraint', 'range-constraint'].flatMap(type => Array(10).fill(null).map((_, i) => new Experiment({
  NAME: `${type}-${i}`,
  EXPERIMENT: 'scaling',
  SEED: 'e8b2018ce02e735c266aa28e8254d05c17c9461b0114aefb01359fc3818a0376',
  ISSUER_SECRET: '',

  WARMUP_STUDIES: ['qualifier', 'disqualifier'].includes(type) ? i + 1 : 1,
  WARMUP_PARTICIPANTS: (type === 'disqualifier' ? 2 : 1) * WARMUP_PARTICIPANTS,

  STUDIES: ['qualifier', 'disqualifier'].includes(type) ? i + 1 : 1,
  PARTICIPANTS: (type === 'disqualifier' ? 2 : 1) * PARTICIPANTS,

  DISTRIBUTIONS: {
    reward: ['uniform', 1, 15],
    attribute: ['uniform', 0, 4294967295]
  },

  generateWorkload(progress, config, prng) {
    const issuer = new Issuer(['set-constraint', 'range-constraint'].includes(type) ? i : 0, Buffer.from(config.SEED, 'hex'));
    faker.seed(prng.integer(0, Number.MAX_SAFE_INTEGER));

    const attributes = Array(issuer.attributes).fill(null).map(() => {
      if (type === 'range-constraint') {
        const bounds = [
          Math.floor(this[`attributeDistribution`]()),
          Math.floor(this[`attributeDistribution`]())
        ].sort();
        return bounds;
      } else {
        const elements = new Set(Array(SET_SIZE).fill(null).map(() => Math.floor(this[`attributeDistribution`]())));
        return [elements.size, [...elements]];
      }
    });

    // init organizer
    const organizerSeed = createHash('sha256').update('organizer@example.org').digest();
    const organizer = {
      id: 'organizer@example.org',
      seed: organizerSeed.toString('base64url')
    };
    organizer.credential = new Organizer(organizer.id, issuer.publicKey, organizerSeed);

    const workload = { length: 0, studies: [], ledger: [] };

    for (const phase of ['WARMUP_', '']) {
      const phaseName = phase.toLowerCase().replace(/[^a-z]*/g, '')

      // init study set
      let bar = progress.start('workload', 'init studies', config.STUDIES);
      const studies = Array(config.STUDIES).fill(null).map(() => {
        const resource = new Resource(null, '', '', '', '', 1, false, '', [], [], []);
        const webBased = prng.bool();
        bar.increment();

        return {
          id: resource.id,
          organizer: `organizer@example.org`,
          reward: Math.round(this.rewardDistribution()),
          qualifier: [],
          constraints: [],
          disqualifier: [],
          name: faker.lorem.sentence(),
          abstract: faker.lorem.sentences(2),
          description: faker.lorem.paragraph(),
          duration: faker.lorem.words(3),
          webBased: webBased,
          studyURL: webBased ? faker.internet.url() : '',
        };
      });
      bar.stop();

      // strip targetStudy and add constraints
      const targetStudy = studies.pop();
      targetStudy.qualifier = type === 'qualifier' ? studies.map(e => e.id) : [];
      targetStudy.disqualifier = type === 'disqualifier' ? studies.map(e => e.id) : [];
      targetStudy.numQualifiers = targetStudy.qualifier.length;
      targetStudy.numDisqualifiers = targetStudy.disqualifier.length;
      targetStudy.constraints = attributes.map((attribute, i) => {
        if (type === 'range-constraint') {
          const bounds = [
            prng.integer(attribute[0], attribute[1]),
            prng.integer(attribute[0], attribute[1]),
          ].sort();
          return [i, 'number', bounds];
        } else {
          return [i, 'select', prng.sample(attribute[1], SET_SAMPLE_SIZE)];
        }
      });

      bar = progress.start('workload', 'initialize study objects', config.STUDIES);
      for (const s of [...studies, targetStudy]) {
        const resource = new Resource(
          s.id,
          s.name,
          s.abstract,
          s.description,
          s.duration,
          s.reward,
          s.webBased,
          s.studyURL,
          s.qualifier.map(e => [e, []]),
          s.disqualifier.map(e => [e, []]),
          s.constraints
        );
        s.signed = Buffer.from(organizer.credential.signResource(resource)).toString('base64url');
        s.resource = resource;
        bar.increment();
      }
      bar.stop();

      // init participants set
      bar = progress.start('workload', `init ${phaseName} participants`, config[`${phase}PARTICIPANTS`]);
      const participants = Array(config[`${phase}PARTICIPANTS`]).fill(null).map((_, i) => {
        bar.increment();
        const id = `participant-${phase}-${i}@example.org`;
        const seed = createHash('sha256').update(id).digest();
        const user = {
          id: id,
          seed: seed.toString('base64url'),
          attributes: targetStudy.constraints.map((constraint) => {
            if (type === 'range-constraint') {
              return prng.integer(constraint[2][0], constraint[2][1]);
            } else {
              return prng.pick(constraint[2]);
            }
          })
        };

        // already register participants
        user.credential = new Participant(
          id,
          new Uint32Array(user.attributes.map(e => Number(e))),
          issuer.ledgerVerificationKey
        );
        const req = user.credential.requestCredential(issuer.publicKey, issuer.verificationKey, seed);
        const res = issuer.issueCredential(req);
        user.credential.retrieveCredential(res);

        return user;
      });
      bar.stop();

      // prepare participations
      const participations = [];
      let dummyParticipants = [];

      if (type === 'disqualifier') {
        dummyParticipants = participants.splice(phaseName === 'warmup' ? WARMUP_PARTICIPANTS : PARTICIPANTS);

        // bootstrap participations of dummy participants in disqualifier
        for (const participant of dummyParticipants) {
          for (const study of studies) {
            const entry = issuer.bootstrapLedger(participant.credential, organizerSeed, study.resource, prng.uuid4());
            workload.ledger.push(entry.serialize());
          }
        }
      } else if (type === 'qualifier') {
        // bootstrap participations of participants in qualifier studies
        for (const participant of participants) {
          for (const study of studies) {
            const entry = issuer.bootstrapLedger(participant.credential, organizerSeed, study.resource, prng.uuid4());
            workload.ledger.push(entry.serialize());
          }
        }
      }

      // let participants participate in target study
      participations.push(...participants.map((participant) => {
        return [ participant.id, targetStudy.id, prng.uuid4() ];
      }));

      // postprocessing
      workload[`${phase}PAYOUTS`] = [];
      workload[`${phase}PARTICIPANTS`] = [...dummyParticipants, ...participants].map((user) => {
        if (user.credential) {
          user.credential = Buffer.from(user.credential.serialize()).toString('base64url');
        }
        return user;
      });
      workload[`${phase}PARTICIPATIONS`] = participations;

      workload.studies.push(...studies, targetStudy);
      workload.length += participants.length + participations.length;
    }

    workload.issuer = Buffer.from(issuer.serialize()).toString('base64url');
    workload.organizers = [{
      ...organizer,
      credential: Buffer.from(organizer.credential.serialize()).toString('base64url')
    }];
    workload.ledger = workload.ledger.map(entry => Buffer.from(entry).toString('base64url'));
    workload.studies.forEach((study) => {
      study.resource.updateReferences(issuer);
      study.object = Buffer.from(study.resource.serialize()).toString('base64url');
      delete study.resource;
    });

    return workload;
  },

  filterWorkload(workload, WORKLOAD_SIZE) {
    if (WORKLOAD_SIZE === 'PETS25_MINIMAL') {
      // minimal workload for fast functionality testing
      workload['WARMUP_PARTICIPATIONS'] = [];
      workload['PARTICIPATIONS'] = workload['PARTICIPATIONS'].slice(0, 1);
      workload.length = 4;
    } else if (WORKLOAD_SIZE === 'PETS25_REDUCED') {
      // smaller workload for less time-consuming evaluation
      workload['WARMUP_PARTICIPATIONS'] = workload['WARMUP_PARTICIPATIONS'].slice(0, 2)
      workload['PARTICIPATIONS'] = workload['PARTICIPATIONS'].slice(0, 4);
      workload.length = 12;
    } else {
      return workload;
    }

    workload['PARTICIPANTS'] = workload['PARTICIPANTS'].filter(e => workload['PARTICIPATIONS'].some(p => p.id === e[0]));
    workload['WARMUP_PARTICIPANTS'] = workload['WARMUP_PARTICIPANTS'].filter(e => workload['WARMUP_PARTICIPATIONS'].some(p => p.id === e[0]));

    return workload;
  },

  filterRecord: `job === 'participations'`
})));
