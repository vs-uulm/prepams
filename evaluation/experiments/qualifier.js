const v8 = require('v8');
const path = require('path');
const Prob = require('prob.js');
const fs = require('fs/promises');
const { MersenneTwister19937 } = require('random-js');
const { Resource } = require('prepams-shared');
const { createHash } = require('crypto');

module.exports = Array(13).fill(null).map((_, i) => ({
  name: `qualifier-${i}`,
  DIR: `${__dirname}/../results/qualifier-${i}`,
  PRNG_SEED: 'e8b2018ce02e735c266aa28e8254d05c17c9461b0114aefb01359fc3818a0376',
  ISSUER_SECRET: 'NEWr88-Nhq7i3-doHZ5UO0O19d5yEHL9sUXkb5s9VXWABjlIRFA4eGxrM3hfODVWT29JbXJIdndqRFIyWnpPb083Z29HZm1uWDh3NHI0dUpBQzZGZVBlMktfZTNhZm1yd0ZDZzVfX2UtLUpoMlZtcHZQVUI0cEM5elFaaU5rUnF5aEpRM2VhRHZ5bDliWlRiLUpvemhfZU1mZnJXcVhJbHNLRnFDVWhkS2h3TGtxTHI0SkpsZWJzRi1tTHRKbHAtemZ0UEdqS25FRjY3TWhlRVd1ZFV6S0hxZnd1dWgtekFjU1d4ZnFZYlBZWFlwWHZHVklYaDRKZkoycVpicDFPOGh0c2hsSVRWdk1heWRiUmQ2aGZfZFBFakh2T1Ruak92UVhYMF9iaThOdzRKMVJBbTRMTWpxalBWZDB3NU9JaFgtN2VLVXNZcXRYZVJhRnBHSmNIaVRlakdrZEZSdWk3TVFEY0N0Y3BVVDFKMV9jV1JmdTd5VnVXWlpLVkx5SVlrTXJwdUNRXzdxeFZ2d1lKVXZsc29QRlVua2cxa0RMa0pRRnVqSUkyWmRZWUVLdXdpT3dBSnFBU3lYWF9UTE5WdEVKTU8wYVJnNkNjeTkzN2lCeWdmemJkRFZDZ2tMZllEVUFUS2J4T3lMMWktN0V5LXJzX0VLZHJzYmdvNFB2ZENvSEo2c051VUxZT3A4azhESjc1ZENHYldCeF9SNEctQlVXdlJjMVYyX2U2VXgzdHgwUVVDUzhITlo1NHdVWUJfM3BSWE1tMTIzVXhLTEc4OVBucmxXU0lXSmNQWjVMNktJRkxvZEpaczBkSlEyeTRqTjZYek40NmMtckpRaG5kUi1qUHN1dVZSWVFOTTlqNFFhVDQ3Q2dvczdWTlhWNGdvME11TzM1YkljZXJVQ1Utc0pWdmMzTnFBZUd2VHFSclJValE5RmY4M3hFZzZSSFhjY0NQc0tzajkyejJ0OXNyWDhTUlE5OW1iZHAta0FWMm9tWEJ5LUpYZVVXRVJKcmJYQnU0Y2VlSmV5V19Oc3FqLVcxcUJ3cHYwdGd6bGtwZHljSSt0Y2V6OVBrZTYwZ0lrNEJTVXgyUnFZTm5LSXdBdHVDY0d3UWg1cnZzSGhv',

  STUDIES: i + 1,
  PARTICIPANTS: 30,

  rewardDistribution: Prob.uniform(1, 15),

  generateWorkload(bar, config) {
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

    // init organizer
    const organizer = {
      id: 'organizer@example.org',
      seed: createHash('sha256').update('organizer@example.org').digest('hex'),
    };

    // init study set
    bar.start('workload', 'init studies', config.STUDIES);
    const studies = Array(config.STUDIES).fill(null).map(() => {
      const reward = Math.round(config.rewardDistribution(next));

      bar.increment();
      return {
        id: new Resource(reward).id,
        organizer: organizer.id,
        reward: reward,
        qualifier: [],
        numQualifiers: 0,
        disqualifier: [],
        numDisqualifiers: 0
      };
    });
    bar.stop();

    // set qualifiers
    const study = studies.pop();
    study.qualifier = new Set(studies);
    study.numQualifiers = studies.length;

    // set participations
    const participations = [
      ...participants.flatMap((participant) => studies.map((study) => {
        participant.participations.add(study);
        return [ participant.id, study.id ];
      })),
      ...participants.map((participant) => {
        participant.participations.add(study);
        return [ participant.id, study.id ];
      })
    ];

    // postprocessing
    study.qualifier = [...study.qualifier].map(e => e.id);
    study.disqualifier = [...study.disqualifier].map(e => e.id);

    const payouts = [];

    const workload = {
      issuer: config.ISSUER_SECRET,
      organizers: [organizer],
      participants: participants.map(e => e.id),
      studies: [...studies, study],
      participations,
      payouts
    };

    return workload;
  },

  async prepareWorkload(bar) {
    let workload;
    try {
      const buf = await fs.readFile(path.join(this.DIR, 'workload.bin'));
      workload = v8.deserialize(buf);
      console.log('[info] loaded previously generated workload');
    } catch {
      workload = this.generateWorkload(bar, this);
      await fs.writeFile(path.join(this.DIR, 'workload.bin'), v8.serialize(workload));
    }
    return workload;
  },

  filterRecord(job, row, i) {
    if (job !== 'participations') {
      return true;
    }

    return this.STUDIES === 1 || i > this.PARTICIPANTS * this.STUDIES;
  }
}));
