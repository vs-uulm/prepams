const { init, Issuer, Organizer, Participant, Resource } = require('prepams-shared');
const { faker } = require('@faker-js/faker');
const { createHash } = require('crypto');
const { sample } = require('random-js');

const v8 = require('v8');
const path = require('path');
const fs = require('fs/promises');
const { MersenneTwister19937, sample } = require('random-js');
const { Resource } = require('prepams-shared');
const { createHash } = require('crypto');

generateWorkload(progress, config, prng) {
  const issuer = Issuer.deserialize(Buffer.from(config.ISSUER_SECRET, 'base64url'), []);
  faker.seed(prng.integer(0, Number.MAX_SAFE_INTEGER));

  // init organizer set
  const organizers = {};

  // init study set
  bar = progress.start('workload', 'init studies', config.STUDIES);
  const studies = Array(config.STUDIES).fill(null).map(() => {
    const reward = Math.round(this.rewardDistribution());

    const organizer = `organizer-${Math.abs(Math.round(this.organizerDistribution()))}@example.org`;
    if (!organizers[organizer]) {
      const seed = createHash('sha256').update(organizer).digest('base64url');
      organizers[organizer] = {
        id: organizer,
        seed: seed,
        credential: new Organizer(organizer, issuer.publicKey, Buffer.from(seed, 'base64url'))
      };
    }

    const numQualifiers = this.qualifierDistribution() - 1;
    const numDisqualifiers = this.disqualifierDistribution() - 1;
    const numConstraints = this.constraintsDistribution() - 1;

    bar.increment();

    const webBased = prng.bool();
    const resource = new Resource(null, '', '', '', '', reward, false, '', [], [], []);

    return {
      id: resource.id,
      organizer: organizer,
      reward: reward, 
      numQualifiers,
      numDisqualifiers,
      name: faker.lorem.sentence(),
      abstract: faker.lorem.sentences(2),
      description: faker.lorem.paragraph(),
      duration: faker.lorem.words(3),
      webBased: webBased,
      studyURL: webBased ? faker.internet.url() : '',
      constraints: Array(numConstraints).fill(null).map(() => {
        const attribute = this.constraintDistribution() - 1;
        switch (attribute) {
          case 0:
          case 3:
          case 4: {
            const stdev = config.DISTRIBUTIONS[`attribute${attribute}`][2];
            const bounds = [
              this[`attribute${attribute}Distribution`](),
              this[`attribute${attribute}Distribution`]()
            ].sort();
            bounds[0] -= stdev;
            bounds[1] += stdev;
            return [0, 'number', bounds.map(e => Math.round(e))];
          }

          case 1:
          case 2: {
            const count = config.DISTRIBUTIONS[`attribute${attribute}`][2];
            const set = new Set(Array(count).fill(null).map(e => prng.integer(1, count) - 1));
            return [0, 'select', [...set]];
          }
        }
      })
    };
  });
  bar.stop();

  // sample qualifiers
  bar = progress.start('workload', 'sample qualifiers', config.STUDIES);
  for (const study of studies) {
    const qualifier = new Set();

    while (qualifier.size < study.numQualifiers) {
      const candidate = studies[this.studyReferenceDistribution() - 1];
      if (candidate !== study) {
        qualifier.add(candidate);
      }
    }
    
    study.qualifier = qualifier;
    bar.increment();
  }
  bar.stop();

  // sample disqualifiers
  bar = progress.start('workload', 'sample disqualifiers', config.STUDIES);
  for (const study of Object.values(studies)) {
    const disqualifier = new Set();

    // run a bfs to find the set of studies that should not become a disqualifier
    const conflicts = new Set();
    const queue = [study];
    while (queue.length > 0) {
      const s = queue.shift();

      if (conflicts.has(s)) {
        continue;
      }

      conflicts.add(s);
      queue.push(...s.qualifier);
    }

    study.numDisqualifiers = Math.min(Object.values(studies).filter(e => !conflicts.has(e) && e !== study).length, study.numDisqualifiers);
    while (disqualifier.size < study.numDisqualifiers) {
      const candidate = studies[this.studyReferenceDistribution() - 1];
      if (candidate !== study && !conflicts.has(candidate)) {
        disqualifier.add(candidate);
      }
    }

    study.conflicts = conflicts;
    study.disqualifier = disqualifier;
    bar.increment();
  }
  bar.stop();
    
  bar = progress.start('workload', 'initialize study objects', config.STUDIES);
  for (const study of Object.values(studies)) {
    study.conflicts = [...study.conflicts].map(e => e.id);
    study.qualifier = [...study.qualifier].map(e => e.id);
    study.disqualifier = [...study.disqualifier].map(e => e.id);

    const resource = new Resource(
      study.id,
      study.name,
      study.abstract,
      study.description,
      study.duration,
      study.reward,
      study.webBased,
      study.studyURL,
      study.qualifier.map(e => [e, []]),
      study.disqualifier.map(e => [e, []]),
      study.constraints
    );
    study.signed = Buffer.from(organizers[study.organizer].credential.signResource(resource)).toString('base64url');
    study.object = Buffer.from(resource.serialize()).toString('base64url');
    bar.increment();
  }

  bar.stop();

  const studyMap = Object.fromEntries(studies.map(e => [e.id, e]));

  const workload = {
    issuer: config.ISSUER_SECRET,
    organizers: Object.values(organizers),
    studies: studies,
    length: 0
  };

  for (const phase of ['WARMUP_', '']) {
    const phaseName = phase.toLowerCase().replace(/[^a-z]*/g, '')
    // init participants set
    let bar = progress.start('workload', `init ${phaseName} participants`, config[`${phase}PARTICIPANTS`]);
    const participants = Array(config[`${phase}PARTICIPANTS`]).fill(null).map((_, i) => {
      bar.increment();
      const id = faker.internet.email();
      return {
        id: id,
        seed: createHash('sha256').update(id).digest('base64url'),
        attributes: [
          Math.round(this.attribute0Distribution()),
          Math.round(this.attribute1Distribution()),
          Math.round(this.attribute2Distribution()),
          Math.round(this.attribute3Distribution()),
          Math.round(this.attribute4Distribution())
        ],
        participations: new Set()
      };
    });
    bar.stop();

    // sample participations
    bar = progress.start('workload', `sample ${phaseName} participations`, config[`${phase}PARTICIPATIONS`]);
    const participations = Array(config[`${phase}PARTICIPATIONS`]).fill(null).map(() => {
      while (true) {
        const participant = participants[this[`${phaseName}participantsDistribution`]() - 1];
        const study = studies[this.studyReferenceDistribution() - 1];

        // redraw if participant already participated in this study
        if (participant.participations.has(study)) {
          continue;
        }

        // redraw if participant is not qualified for this study
        if (!study.qualifier.every(e => participant.participations.has(e))) {
          continue;
        }
        if (study.disqualifier.some(e => participant.participations.has(e))) {
          continue;
        }
        if (!study.constraints.every(e => {
          if (e[1] === 'select') {
            return e[2].includes(participant.attributes[e[0]]);
          } else {
            return e[2][0] <= participant.attributes[e[0]] && e[2][0] >= participant.attributes[e[0]] ;
          }
        })) {
          continue;
        }

        // otherwise schedule participation
        participant.participations.add(study.id);
        bar.increment();

        return [ participant.id, study.id, prng.uuid4() ];
      }
    });
    bar.stop();

    // sample payouts
    bar = progress.start('workload', `sample ${phaseName} payouts`, config[`${phase}PAYOUTS`]);
    const payouts = Array(config[`${phase}PAYOUTS`]).fill(null).map(() => {
      while (true) {
        const participant = participants[this[`${phaseName}participantsDistribution`]() - 1];
        const rewards = Math.abs(Math.round(this.payoutRewardDistribution()));

        // redraw if the participant can't request this payout
        if (rewards === 0 || rewards > participant.participations.size) {
          continue;
        }

        const payout = sample(this.prng, [...participant.participations], rewards);
        participant.participations.clear();

        bar.increment();
        return [ participant.id, payout.reduce((a, e) => a + studyMap[e].reward, 0) ];
      }
    });
    bar.stop();

    // postprocessing
    bar = progress.start('workload', 'postprocessing', config[`${phase}PARTICIPANTS`]);
    for (const participant of participants) {
      delete participant.participations;
      bar.increment();
    }
    bar.stop();

    workload[`${phase}PARTICIPANTS`] = participants;
    workload[`${phase}PARTICIPATIONS`] = participations;
    workload[`${phase}PAYOUTS`] = payouts;
    workload.length += participants.length + participations.length + payouts.length;
  }

  // postprocessing
  bar = progress.start('workload', 'postprocessing', config.STUDIES + Object.keys(organizers).length);
  for (const study of studies) {
    delete study.conflicts;
    bar.increment();
  }
  for (const organizer of Object.values(organizers)) {
    organizer.credential = Buffer.from(organizer.credential.serialize()).toString('base64url');
    bar.increment();
  }
  bar.stop();

  return workload;
}

module.exports = async function prepareWorkload(bar, config) {
  let workload;
  try {
    const buf = await fs.readFile(path.join(config.DIR, 'workload.bin'));
    workload = v8.deserialize(buf);
    console.log('[info] loaded previously generated workload');
  } catch {
    workload = generateWorkload(bar, config);
    await fs.writeFile(path.join(config.DIR, 'workload.bin'), v8.serialize(workload));
  }
  return workload;
};
