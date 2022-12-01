const v8 = require('v8');
const path = require('path');
const fs = require('fs/promises');
const { MersenneTwister19937, sample } = require('random-js');
const { Resource } = require('prepams-shared');
const { createHash } = require('crypto');

function generateWorkload(bar, config) {
  const seed = [...new Uint32Array(new Uint8Array(Buffer.from(config.PRNG_SEED, 'hex')).buffer)];
  const prng = MersenneTwister19937.seedWithArray(seed);
  const next = () => prng.next();

  // init participants set
  bar.start('workload', 'init participants', config.PARTICIPANTS);
  const participants = Array(config.PARTICIPANTS).fill(null).map((_, i) => {
    bar.increment();
    const id = `participant-${i}@example.org`;
    return {
      id: id,
      seed: createHash('sha256').update(id).digest('hex'),
      participations: new Set()
    };
  });
  bar.stop();

  // init organizer set
  const organizers = new Map();

  // init study set
  bar.start('workload', 'init studies', config.STUDIES);
  const studies = Array(config.STUDIES).fill(null).map(() => {
    const reward = Math.round(config.rewardDistribution(next));

    const organizer = `organizer-${Math.abs(Math.round(config.organizerDistribution(next)))}@example.org`;
    if (!organizers.has(organizer)) {
      organizers.set(organizer, {
        id: organizer,
        seed: createHash('sha256').update(organizer).digest('hex'),
      });
    }

    const numQualifiers = config.qualifierDistribution(next) - 1;
    const numDisqualifiers = config.qualifierDistribution(next) - 1;

    bar.increment();
    return {
      id: new Resource(reward).id,
      organizer: organizer,
      reward: reward, 
      numQualifiers,
      numDisqualifiers
    };
  });
  bar.stop();

  // sample qualifiers
  bar.start('workload', 'sample qualifiers', config.STUDIES);
  for (const study of studies) {
    const qualifier = new Set();

    while (qualifier.size < study.numQualifiers) {
      const candidate = studies[config.studyReferenceDistribution(next) - 1];
      if (candidate !== study) {
        qualifier.add(candidate);
      }
    }
    
    study.qualifier = qualifier;
    bar.increment();
  }
  bar.stop();

  // sample disqualifiers
  bar.start('workload', 'sample disqualifiers', config.STUDIES);
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

    while (disqualifier.size < study.numDisqualifiers) {
      const candidate = studies[config.studyReferenceDistribution(next) - 1];
      if (candidate !== study && !conflicts.has(candidate)) {
        disqualifier.add(candidate);
      }
    }
    
    study.conflicts = conflicts;
    study.disqualifier = disqualifier;
    bar.increment();
  }
  bar.stop();

  // sample participations
  bar.start('workload', 'sample participations', config.PARTICIPATIONS);
  const participations = Array(config.PARTICIPATIONS).fill(null).map(() => {
    while (true) {
      const participant = participants[config.participantsDistribution(next) - 1];
      const study = studies[config.studyReferenceDistribution(next) - 1];

      // redraw if participant already participated in this study
      if (participant.participations.has(study)) {
        continue;
      }

      // redraw if participant is not qualified for this study
      if (![...study.qualifier].every(e => participant.participations.has(e))) {
        continue;
      }
      if ([...study.disqualifier].some(e => participant.participations.has(e))) {
        continue;
      }

      participant.participations.add(study);
      bar.increment();
      return [ participant.id, study.id ];
    }
  });
  bar.stop();

  // sample payouts
  bar.start('workload', 'sample payouts', config.PAYOUTS);
  const payouts = Array(config.PAYOUTS).fill(null).map(() => {
    while (true) {
      const participant = participants[config.participantsDistribution(next) - 1];
      const rewards = Math.abs(Math.round(config.payoutRewardDistribution(next)));

      // redraw if the participant can't request this payout
      if (rewards === 0 || rewards > participant.participations.size) {
        continue;
      }

      const payout = sample(prng, [...participant.participations], rewards);
      payout.forEach(e => participant.participations.delete(e));

      bar.increment();
      return [ participant.id, payout.reduce((a, e) => a + e.reward, 0) ];
    }
  });
  bar.stop();

  // sample payouts
  bar.start('workload', 'postprocessing', config.STUDIES);
  for (const study of studies) {
    study.qualifier = [...study.qualifier].map(e => e.id);
    study.disqualifier = [...study.disqualifier].map(e => e.id);
    // delete study.conflicts;
    study.conflicts = [...study.conflicts].map(e => e.id);
    bar.increment();
  }
  bar.stop();

  const workload = {
    issuer: config.ISSUER_SECRET,
    organizers: [...organizers.values()],
    participants: participants.map(e => e.id),
    studies,
    participations,
    payouts
  };

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
