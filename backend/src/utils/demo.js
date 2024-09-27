const { Participant, Organizer, Resource, SignedResource } = require('prepams-shared');

const demoIdentities = [{
  id: 'participant1@example.org',
  role: 'participant',
  attributes: {
    'year of birth': 1995,
    'handedness': 'right'
  },
  seed: '9a5100934e010018886a9ef74d7f14a0221fb540f313948d650bcf4cb56052b9',
  key: {
    alg: 'A256GCM',
    ext: true,
    k: 'hXXqAvY9t5BUODlo650rlQs3vnd-IcO_ZIIGicIeaTo',
    key_ops: ['encrypt', 'decrypt'],
    kty: 'oct'
  }
}, {
  id: 'participant2@example.org',
  role: 'participant',
  attributes: {
    'year of birth': 1887,
    'handedness': 'right'
  },
  seed: 'fbd4ae224d9024fc6b26ba66522fa54e1cdbe5414dd2c93191f9737cfeeadd30',
  key: {
    alg: 'A256GCM',
    ext: true,
    k: 'tohso7DBq7RSFw8NsQJwOYxd5Qvup18Ov4mCxLwUXXo',
    key_ops: ['encrypt', 'decrypt'],
    kty: 'oct'
  }
}, {
  id: 'participant3@example.org',
  role: 'participant',
  attributes: {
    'year of birth': 2001,
    'handedness': 'left'
  },
  seed: '03b900a7ee37f605f6f67d1d22306657b5a42aecef2b2d306dd2a1148b3ca79d',
  key: {
    alg: 'A256GCM',
    ext: true,
    k: 'mgqY_uDT63c-0CiGnT0LIdw8KjtK5Qx4_bTBgILQkTk',
    key_ops: ['encrypt', 'decrypt'],
    kty: 'oct'
  }
}, {
  id: 'organizer1@example.org',
  role: 'organizer',
  seed: 'c9cda9663257b43888cf0ec5ea4827dd161ae48a6912612f755670bb777023ad',
  key: {
    alg: 'A256GCM',
    ext: true,
    k: 'g3wXrjcyaG0lhz1qey1r0t5MR1LJZLpZ-Be1aaVsnHg',
    key_ops: [
      'encrypt',
      'decrypt'
    ],
    kty: 'oct'
  },
  studies: [{
    id: 'MZQOjuGaFLfCMzVTuu2xYabC03kilWsRxPCPz9Rl7Cg',
    name: 'Chatbot-based Training for Stress Reduction and Health Benefits',
    abstract: 'The purpose of this study is to investigate the extent to which this chat-bot based training can reduce stress and improve health at the same time.',
    description: '-',
    duration: '3 weeks',
    reward: 45,
    webBased: false,
    studyUrl: null,
    qualifier: [],
    disqualifier: [],
    constraints: [[ 0, "number", [ 1990, 2000 ] ]]
  }, {
    id: 'CqDYCcS9vnZSKGDNIkEql1bfOBluvUvchrdgQ6Ijv0c',
    name: 'COVID-19, Personality and Social Media Usage',
    abstract: 'Earn 5 easy credits by completing our questionnaire on social media use during the COVID-19 pandemic and personality traits.',
    description: '-',
    duration: '30 minutes',
    reward: 5,
    webBased: false,
    studyUrl: null,
    qualifier: [],
    disqualifier: ['MZQOjuGaFLfCMzVTuu2xYabC03kilWsRxPCPz9Rl7Cg'],
    constraints: []
  }, {
    id: 'Xp5UmTZd-1hyhvJ7Ct9hv1amLyhaJPBi8mdmvcZwGi8',
    name: 'COVID-19, Personality and Social Media Usage - Follow-Up',
    abstract: 'Follow-up questionnaire on social media use during the continuing COVID-19 pandemic.',
    description: '-',
    duration: '25 minutes',
    reward: 5,
    webBased: false,
    studyUrl: null,
    qualifier: ['CqDYCcS9vnZSKGDNIkEql1bfOBluvUvchrdgQ6Ijv0c'],
    disqualifier: [],
    constraints: []
  }, {
    id: 'jzL5j7iQXZjn29s1iygahCd_fHI_gBSJWH7oFiH3Q1s',
    name: 'Personality Traits Example Survey',
    abstract: 'A web-based survey demo.',
    description: '-',
    duration: '5 minutes',
    reward: 10,
    webBased: true,
    studyUrl: `https://${process.env['SURVEY_HOST']}`,
    qualifier: [],
    disqualifier: [],
    constraints: []
  }]
}, {
  id: 'organizer2@example.org',
  role: 'organizer',
  seed: 'bacf09abb522825aa497f173dfa300e29fc88e24fb9ac17aa1b6a1648f8856cc',
  key: {
      alg: 'A256GCM',
      ext: true,
      k: 'O6oSKK-2ee79RCrxKu-twzxAI5lnLc8OZLIOsg9Cf9I',
      key_ops: [
          'encrypt',
          'decrypt'
      ],
      kty: 'oct'
  },
  studies: []
}];

module.exports = {
  demoIdentities,

  async populateDemoData(db, issuer, ATTRIBUTES) {
    // create demo identities
    const [rows, issuedSignatures] = await Promise.all([
      db.all('SELECT * FROM users'),
      db.all('SELECT signature FROM issued')
    ]);

    for (const identity of demoIdentities) {
      const id = identity.id;
      const pk = issuer.publicKey;
      const vk = issuer.verificationKey;
      const lvk = issuer.ledgerVerificationKey;
      const seed = Buffer.from(identity.seed, 'hex');
      const role = identity.role;
      const row = rows.find(e => e.id === identity.id);
      const attributes = role === 'participant' ? new Uint32Array(ATTRIBUTES.map(a => a[1] === 'select' ? a[2].indexOf(identity.attributes[a[0]]) : identity.attributes[a[0]])) : [];

      if (row && !identity.state) {
        if (role === 'participant') {
          const participant = new Participant(id, attributes, lvk);
          participant.requestCredential(pk, vk, seed);

          for (const { signature } of issuedSignatures) {
            try {
              await participant.retrieveCredential(await signature);
              identity.state = Buffer.from(participant.serialize()).toString('hex');
              break;
            } catch {
              // ignore
            }
          }
        } else {
          const user = new Organizer(id, pk, seed);
          identity.state = Buffer.from(user.serialize()).toString('hex');
        }
        continue;
      }

      if (role === 'participant') {
        // emulate client
        const user = new Participant(id, attributes, lvk);
        const request = user.requestCredential(pk, vk, seed);

        // emulate service
        const signature = issuer.issueCredential(request);
        await db.run('INSERT INTO users (id, role) VALUES (?, ?)', id, 'participant');
        await db.run('INSERT INTO issued (signature) VALUES (?)', signature);

        // emulate client
        user.retrieveCredential(signature);
        identity.state = Buffer.from(user.serialize()).toString('hex');
      } else {
        // emulate client
        const user = new Organizer(id, pk, seed);
        // emulate service
        await db.run('INSERT INTO users (id, role, publicKey) VALUES (?, ?, ?)', id, 'organizer', user.publicKey);
        identity.state = Buffer.from(user.serialize()).toString('hex');

        // create studies if necessary
        for (const study of identity.studies) {
          const signedResource = SignedResource.deserialize(
            user.signResource(
              new Resource(
                study.id,
                study.name || '',
                study.abstract || '',
                study.description || '',
                study.duration || '',
                study.reward,
                study.webBased,
                study.studyUrl || '',
                study.qualifier.map(e => [e, []]),
                study.disqualifier.map(e => [e, []]),
                study.constraints
              )
            )
          );
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
              constraints,
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
              :constraints,
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
            ':constraints': JSON.stringify(resource.constraints),
            ':webBased': resource.webBased,
            ':studyURL': resource.studyUrl,
            ':signature': signedResource.signature
          });
        }
      }
    }

    return demoIdentities;
  }
}
