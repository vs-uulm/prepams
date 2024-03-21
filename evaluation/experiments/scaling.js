const { init, Issuer, Organizer, Resource } = require('prepams-shared');
const { createHash } = require('crypto');
const { faker } = require('@faker-js/faker');

const Experiment = require('../src/experiment');
init();

const PARTICIPANTS = 10;

const SET_SIZE = 128;
const SET_SAMPLE_SIZE = 32;

// module.exports = ['range-constraint'].flatMap(type => Array(13).fill(null).map((_, i) => new Experiment({
module.exports = ['qualifier', 'disqualifier', 'set-constraint', 'range-constraint'].flatMap(type => Array(13).fill(null).map((_, i) => new Experiment({
  NAME: `${type}-${i}`,
  SEED: 'e8b2018ce02e735c266aa28e8254d05c17c9461b0114aefb01359fc3818a0376',
  ISSUER_SECRET: '',

  STUDIES: ['qualifier', 'disqualifier'].includes(type) ? i + 1 : 1,
  PARTICIPANTS: (type === 'disqualifier' ? 2 : 1) * PARTICIPANTS,
  WARMUP_PARTICIPANTS: PARTICIPANTS,

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
    const organizer = {
      id: 'organizer@example.org',
      seed: createHash('sha256').update('organizer@example.org').digest(),
    };
    organizer.credential = new Organizer(organizer.id, issuer.publicKey, organizer.seed);

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
        disqualifier: [],
        name: faker.lorem.sentence(),
        abstract: faker.lorem.sentences(2),
        description: faker.lorem.paragraph(),
        duration: faker.lorem.words(3),
        webBased: webBased,
        studyURL: webBased ? faker.internet.url() : '',
        constraints: attributes.map((attribute, i) => {
          if (type === 'range-constraint') {
            const bounds = [
              prng.integer(attribute[0], attribute[1]),
              prng.integer(attribute[0], attribute[1]),
            ].sort();
            return [i, 'number', bounds];
          } else {
            return [i, 'select', prng.sample(attribute[1], SET_SAMPLE_SIZE)];
          }
        })
      };
    });
    bar.stop();

    // set qualifiers
    const study = studies.pop();
    study.qualifier = type === 'qualifier' ? studies.map(e => e.id) : [];
    study.disqualifier = type === 'disqualifier' ? studies.map(e => e.id) : [];
    study.numQualifiers = study.qualifier.length;
    study.numDisqualifiers = study.disqualifier.length;

    bar = progress.start('workload', 'initialize study objects', config.STUDIES);
    for (const s of [...studies, study]) {
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
      s.object = Buffer.from(resource.serialize()).toString('base64url');
      bar.increment();
    }
    bar.stop();

    organizer.credential = Buffer.from(organizer.credential.serialize()).toString('base64url');

    const workload = {
      issuer: Buffer.from(issuer.serialize()).toString('base64'),
      organizers: [organizer],
      studies: [...studies, study],
      length: 0
    };

    for (const phase of ['WARMUP_', '']) {
      const phaseName = phase.toLowerCase().replace(/[^a-z]*/g, '')

      // init participants set
      let bar = progress.start('workload', `init ${phaseName} participants`, config[`${phase}PARTICIPANTS`]);
      const participants = Array(config[`${phase}PARTICIPANTS`]).fill(null).map((_, i) => {
        bar.increment();
        const id = `participant-${phase}-${i}@example.org`;
        return {
          id: id,
          seed: createHash('sha256').update(id).digest('base64url'),
          attributes: study.constraints.map((constraint) => {
            if (type === 'range-constraint') {
              return prng.integer(constraint[2][0], constraint[2][1]);
            } else {
              return prng.pick(constraint[2]);
            }
          })
        };
      });
      bar.stop();

      // set participations
      const participations = [];
      let dummyParticipants = [];
      if (type === 'disqualifier' && phaseName !== 'warmup') {
        dummyParticipants = participants.splice(PARTICIPANTS);
        participations.push(...dummyParticipants.flatMap((participant) => studies.map((study) => {
          return [ participant.id, study.id, prng.uuid4() ];
        })));
      }
      if (type === 'qualifier' || phaseName === 'warmup') {
        participations.push(...participants.flatMap((participant) => studies.map((study) => {
          return [ participant.id, study.id, prng.uuid4() ];
        })));
      }
      if (type === 'qualifier' || phaseName !== 'warmup') {
        participations.push(...participants.map((participant) => {
          return [ participant.id, study.id, prng.uuid4() ];
        }));
      }

      // postprocessing
      workload[`${phase}PAYOUTS`] = [];
      workload[`${phase}PARTICIPANTS`] = [...participants, ...dummyParticipants];
      workload[`${phase}PARTICIPATIONS`] = participations;
      workload.length += participants.length + participations.length;
    }

    return workload;
  },

  filterRecord: `job === 'participations' && (['set-constraint', 'range-constraint'].some(e => workload.startsWith(e)) || i >= ${PARTICIPANTS} * ${i})`
})));
