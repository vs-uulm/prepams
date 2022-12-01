import Vue from 'vue';
import Vuex from 'vuex';

import axios from 'axios';
import QRCode from 'qrcode';

import { Organizer, Participant, Participation, Resource } from 'prepams-shared';

import logo from '@/assets/logo.png';

Vue.use(Vuex);

const encoder = new TextEncoder();
const toHex = a => new Uint8Array(a).reduce((s, b) => s + b.toString(16).padStart(2, '0'), '');
const fromHex = s => new Uint8Array(s.match(/.{1,2}/g).map(b => parseInt(b, 16)));

if (!localStorage.getItem('keys')) {
  // auto login for demo accounts
  localStorage.setItem('keys', '{"participant@example.com":{"alg":"A256GCM","ext":true,"k":"zQLoB2Y1ky55I4oXUO5Xwq9DxYK3eZrZ6idrsK1WYuI","key_ops":["encrypt","decrypt"],"kty":"oct"},"organizer@example.com":{"alg":"A256GCM","ext":true,"k":"FmL6987UgEUc78den-fEHCb0u0g2gOKByAeyhADdNrc","key_ops":["encrypt","decrypt"],"kty":"oct"}}');
  localStorage.setItem('organizer:organizer@example.com', 'e7b132deed7aa7083ec99969a783fb2cbc69a4eb6c643ec0b1700823ee370498712e9d2cbe68498def7183516650e4d10c8617388b4c222e7c77bb424e83f2b4adea9b9c93a2014622d247ac871912deada6f4f3b6e3f72a3d060eed06b9e1e98cd6de6495dc225146857d00d1babb06bdf0bd3bfe6a5bbf41a3718f67941932adf449853a98c079c50f38fb65e645b6f2d6ae2c8f17ef067396510455b47e0b54e17d297464e64af9972a9498364c6200c60e062df4767fff5e3d5eae6a51e48350d3397d5e126cd7d7a89faf72a9393bc2132492da7d950f24aeb7d3003cc9fe000823d3f871362eb4177c511f41fcbcfb8477d8233daa69a671f0daac7c84ce4fa04a5f7d618fa59bb38c94c90fd3ba71a0cf2fc1c664472e6bd942f72fd1806e2cd0209934cf1c1138fe19b3b9a3c5f352a6ff4b625b7fb5e9cad2ef28ac1f3d24094dca3e3fd6850d647bbcda72397ab09f5f46f92313f1dd644c6d31f8f70089e8dc96404a2ce2e945f114163617759c4d182f0e31a765a82440a927c011e1c887c36643301cd02a6f8c97625cb7d4a7e3db004df6a427468de947fe05727a9eaf4ea5510279f11b416ef59de00cacbc74e8e0e25c7cc89d37bafc0c575f1b866ac14248a5ee306568809ff06bbf86e0bb60db54ccc8b870e41f3dd1d6fe40e743c1dc5af3869e210e2f536c60b7330b5cb39f6c983332fe1f7c83434a78bf5c9ffbc25806df2c55229138199b3c3f3e1a6b9f3abca634bd21ba01c4054b5ae11593569f469477170f4296e467b27ee572ffabfca85db8135125b4150d081c0837dc47e2cc77bbbafd3fdfab5551cdf386a782f1d002ff7dfe4566c7cb1b6449b29604cd8e5de0769f762cd62a7150b0668e163d6bbd5865e0dcb85958b8009cc69d7f28ec1a809b77ee255edd0e7db627b72fa4780b24ee955bc6e75fa2aeb7a09c05bc2d89eed2bc1970f93d3c0a37961a705a8e455831d9538dba72697110f603b4031a4e5c8d49bf7c63551077e0857aac5fd46518bf1c762519533bea0f3eec1294d68ca149323f40d05f4df94b40c4083cc24af6055d11d1f1560989c60d68785eccdd24f2f7420c87366fd7b7028819cd35c1ab34e6108dcf8cd0e677b6fb862b6ddd85b39f15b5c8a1251b59a7efd3bb62ef771dda1594b45185a9524f87f7fb4f336caa9dd9b5bd2f9079c6a2');
  localStorage.setItem('participant:participant@example.com', 'a24f3c873dd4c86a506ede56c775050d5c457a21f2e9e84fb71edd425861d8a566843384249d3b73c70cd74597d015f9910515a046b0e9dc4c9b9c159e28f514e43ebb7950b5c6cc13a74cc1768f29d882477147d6cfb5516cac79a0dfbd036da81173efdf08324809072bd8022129b5209caf3abec83958dd436a86effdaa5c1fe6f096ae4737d0fff5e2f9fb7b1531a8d11c55ca8720f13d952104c9fe143e1f46af6360d44b952a852228f0adab4462087f274c30327269f0b54fb76a5fb0dbcf2c2773dc17c54372da923223966661979d116abfec8b42b8a0545ede6014345c28ff2caf32684824e67fb2c607b27149e1627e136a499bfc89cd2302406dbe820deb2c2adf985ad3f5cb194a6f3fbf36f20ac6413eb32c027e35793ed8f3e95e9087247f10efe9f3c43b8989ba93787ec0b629771ed3e903af962c6d4bfc619e7f2be1ff81748f15ad60ff7b1d043e4143c4c2756e32860012e9c297e29a72ce3f97bf95df78e189ccb3652cf6c48fe3eb9e065c1fee5e0a375ccd37b5c7116865b4bc6ef78f124e08b05dc50021e84f99e3e3d1692dfee9bc003371ae33aead5da7bcad9d554d2b489a4548f77bcb629716e74d17c4709cfc36842a60bc6665360d9580c70ab52fb177f2676fe8ff301458ebe43f0503a25aaf341e6fd814310eff1f5501efd792de34dcb000b6e3714744b146762c6b7fccbf5f368aab64ea3b1d6afcc55bfc7446b6ef9d3c9925f532a028a1d6cefb68627de9b4b19071e19c8c6f09133cbbcb986c8802fee8addf09444b5221f432d53376a68cbe10516048bc6846e8ffddcd8a373e69743759925cc9fae9506cc4fa52f0efd1aadc6b71f1e9f33377e6afe82eb627d8f93d90999baad8c8150b2932f3723f5898137849aaee453e3ecd4e599f07a7631b05ddd50fd2d00bda2a66148a09215029189cb1b16191afb51b8cb2d8370e2ff2c3d13fdaa54573d8ae99125b78f4310a4e364009f58ccb51514e051ab7d0675ad8d87ec6fc90697785ff4af795860f12b4091d82dcc0abc78c9bf85921e18bfb21c37761c197e4915802b1b77543397fe99bacec1eaf25f29ff62db48e32a34a1a6afe38cd7ce2de22a24c4856ce52a8b6ebe7cb307e7c60bc53bf57f40549becbb73576c1825c10d06b389f9a449d8acf03545ea3f124182bad451456c1ee29eb9d9f38da9b62da719e9f77337fa30a4503c60943eddf3a03dff71e1bb75392b2a9198698ff5c9342823ba1bb1b79072e141d817f751d640e011f27f232073a0a8ac9428b0275abfe5fe72c48775ab1dcfb9dfd4a09201fd93d180d60a412349bb0cf691d64dbdac53a48f492d37c90e96f00ffa7833357db213af26488541d298bb52e9276aced95da1069a00673fb1a8889777067d3132ed774b6d08de474e5108ff5fe6001154aacbc46e00c3c6047dacddbe3b648cc9cbe27c41e3795ac912c9fe4425239115ba7243d170d17b7d69dceb6d54dc4c52af3408e43817f6855970ea361b8232d8cb36532b05bc1fd5f95cbb53889b24ee0e9c6e9a387b27041352b79a9dda284ee59a968aa859e756966c0dbc1869d0337e82225717906435fdae9a419e2bd9e11a567f8e92f58c103e7daaba424b5bd99df2f82c55d47790c1a3fa02b63c025');
}

const keys = {};
(async () => {
  try {
    for (const [id, key] of Object.entries(JSON.parse(localStorage.getItem('keys')))) {
      keys[id] = await window.crypto.subtle.importKey('jwk', key, { name: 'AES-GCM' }, true, [ 'encrypt', 'decrypt' ])
    }
  } catch {
    // ignore
  }
})()

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

  keys[id] = decryptionKey;
  return new Uint8Array(seed);
}

async function persistCredential(credential) {
  if (!keys[credential.id]) {
    return;
  }

  const data = credential.serializeBinary();
  const iv = window.crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await window.crypto.subtle.encrypt({
    name: 'AES-GCM',
    iv: iv
  }, keys[credential.id], data);

  const blob = toHex(iv) + toHex(ciphertext);
  localStorage.setItem(`${credential.role}:${credential.identity}`, blob);
}

async function loadCredential(id, role, data) {
  try {
    const iv = fromHex(data.slice(0, 24));
    const ciphertext = fromHex(data.slice(24));
    const plaintext = await window.crypto.subtle.decrypt({
      name: 'AES-GCM',
      iv: iv
    }, keys[id], ciphertext);

    switch (role) {
      case 'participant':
        return Participant.deserializeBinary(new Uint8Array(plaintext));
      case 'organizer':
        return Organizer.deserializeBinary(new Uint8Array(plaintext));
      default:
        throw new Error(`${role} not supported`);
    }
  } catch {
    keys[id] = null;
    throw new Error('restore failed');
  }
}

export default new Vuex.Store({
  state: {
    user: null,
    credential: 'Unauthenticated',
    credentials: [
      {
        role: 'none',
        id: 'Unauthenticated'
      },
      ...Object.keys(localStorage)
        .filter(e => e.startsWith('participant:') || e.startsWith('organizer:'))
        .map(e => ({
          id: e.slice(e.indexOf(':') + 1),
          role: e.slice(0, e.indexOf(':'))
        }))
    ]
  },

  mutations: {
    async authenticated(state, user) {
      state.credential = user?.id || 'Unauthenticated';
      localStorage.setItem('credential', state.credential);

      if (user) {
        user.participated = [];
      }
      state.user = user;

      if (user && !state.credentials.find(e => e.id === user.id)) {
        state.credentials.push({
          id: user.id,
          role: user.role
        });
      }

      if (user?.credential && !localStorage.getItem(`${user.role}:${user.id}`)) {
        await persistCredential(user.credential);
      }

      localStorage.setItem('keys', JSON.stringify(Object.fromEntries(await Promise.all(
        Object.keys(keys)
          .filter(e => e && keys[e])
          .map(async (e) => {
            const jwk = await window.crypto.subtle.exportKey('jwk', keys[e]);
            return [e, jwk];
          })
      ))));
    },

    restore(state, restoredState) {
      Object.assign(state, restoredState);
    },

    newBalance(state, { balance, participated }) {
      state.user.balance = balance;
      state.user.participated = participated;
    },

    async lock(state) {
      if (state.user?.id) {
        keys[state.user.id] = null;
      }

      state.user = null;
    }
  },

  actions: {
    async switchAccount(context, id) {
      const role = localStorage.getItem(`participant:${id}`) ? 'participant' : 'organizer';
      const data = localStorage.getItem(`${role}:${id}`);

      if (id === 'Unauthenticated') {
        context.commit('authenticated', null);
        return;
      }

      try {
        const credential = await loadCredential(id, role, data);
        context.commit('authenticated', {
          balance: 0,
          id: credential.id,
          role: credential.role,
          credential: credential
        });
      } catch {
        const e = new Error('incorrect password');
        e.role = role;
        e.id = id;
        throw e;
      }
    },

    recover(context, data) {
      for (const role of [Participant, Organizer]) {
        try {
          const credential = role.deserializeBinary(data);
          context.commit('authenticated', {
            balance: 0,
            id: credential.id,
            role: credential.role,
            credential: credential
          });
          return;
        } catch {
          // ignore
        }
      }

      throw new Error('could not recover account from code');
    },

    async signin(context, { id, password, role }) {
      if (localStorage.getItem(`${role}:${id}`)) {
        await deriveKeys(id, password);
        await context.dispatch('switchAccount', id);
      } else if (role === 'participant') {
        // try to recover account from public issued log
        const seed = await deriveKeys(id, password);
        const participant = new Participant(id);

        const req = await axios.get(`/api/auth/signin?role=${role}`);
        participant.requestCredential(req.data.issuer.publicKey, seed);

        for (const signature of req.data.log) {
          try {
            await participant.retrieveCredential(signature);
            await context.commit('authenticated', {
              balance: 0,
              id: id,
              role: role,
              credential: participant
            });
            return;
          } catch {
            // ignore
          }
        }

        throw new Error('could not recover account');
      } else {
        const seed = await deriveKeys(id, password);
        const req = await axios.get(`/api/auth/signin?role=${role}`);
        const user = new Organizer(id, req.data.issuer.publicKey, seed);

        if (!req.data.publicKeys.includes(user.publicKey)) {
          throw new Error('username or password invalid');
        }

        await context.commit('authenticated', {
          balance: 0,
          id: id,
          role: role,
          credential: user
        });
      }
    },

    async signup(context, { id, password, role }) {
      const seed = await deriveKeys(id, password);
      const issuer = await axios.get('/api/auth/signup');

      let user = null;
      let request = null;
      if (role === 'participant') {
        user = new Participant(id);
        request = user.requestCredential(issuer.data.publicKey, seed);
      } else {
        user = new Organizer(id, issuer.data.publicKey, seed);
        request = {
          id: id,
          publicKey: user.publicKey
        };
      }

      const res = await axios.post(`/api/auth/signup?role=${role}`, request);
      if (role === 'participant') {
        user.retrieveCredential(res.data);
      }

      const state = user.serializeBinary();
      const recoveryCode = await QRCode.toDataURL([{ data: state, mode: 'byte' }], {
        errorCorrectionLevel: 'L'
      });

      const canvas = document.createElement('canvas');
      canvas.height = 940;
      canvas.width = 600;
      
      const ctx = canvas.getContext('2d');
      ctx.fillStyle = 'white';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      ctx.font = 'bold 28px Roboto, sans-serif';
      ctx.fillStyle = 'black';
      ctx.fillText('PrePaMS Restore Code', 100, 50);

      ctx.font = '23px Roboto, sans-serif';
      ctx.textAlign = 'left';
      ctx.fillText(state.identity, 100, 78);

      const img = new Image();
      img.src = logo;
      await new Promise(resolve => img.addEventListener('load', resolve));
      ctx.drawImage(img, 2, 0, 90, 90);

      const qr = new Image();
      qr.src = recoveryCode;
      await new Promise(resolve => qr.addEventListener('load', resolve));
      ctx.drawImage(qr, 25, 115, 550, 550);

      [
        'This QR code is your recovery key.  If you loose access',
        'to your device,  clear your browser data,  or forget  your',
        'password,  you need this code to regain access to your',
        'PrePaMS account.',
        '',
        'Treat this code the same way as you would a password,',
        'because  anyone  with  this code can get access to your',
        'account!',
        '',
        'Hint: Keep a physical copy in your sock drawer to be on',
        '          the safe side :)',
      ].forEach((e, i) => ctx.fillText(e, 18, 710 + 22 * i))

      ctx.font = '16px Roboto, sans-serif';
      ctx.textAlign = 'right';
      ctx.fillStyle = 'grey';
      ctx.fillText(new Date().toJSON().slice(0, 10), 578, 58);

      ctx.font = '16px Roboto, sans-serif';
      ctx.textAlign = 'right';
      ctx.fillStyle = 'grey';
      ctx.fillText(new Date().toJSON().slice(11, 19), 578, 78);

      const recoveryFile = canvas.toDataURL();

      await context.commit('authenticated', {
        balance: 0,
        id: id,
        role: role,
        credential: user
      });

      return { id, recoveryCode, recoveryFile };
    },

    lock(context) {
      context.commit('lock');
    },

    async participate(context, study) {
      try {
        const req = await axios.get(`/api/rewards`);
        const participations = req.data.transactions;

        const resource = Resource.deserialize({
          id: study.id,
          reward: study.reward,
          qualifier: study.qualifier.map(id => ({
            id,
            tags: participations.filter(e => e.id === id).map(e => e.tag)
          })),
          disqualifier: study.disqualifier.map(id => ({
            id,
            tags: participations.filter(e => e.id === id).map(e => e.tag)
          }))
        });

        const participation = context.state.user.credential.participate(resource);

        const iv = window.crypto.getRandomValues(new Uint8Array(12));
        const key = await window.crypto.subtle.generateKey({
          name: 'AES-GCM',
          length: 256
        }, true, ['encrypt']);

        const p = participation.serialize();
        const encoded = participation.serializeBinary();
        const data = await window.crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, encoded);

        const participationKey = toHex(await window.crypto.subtle.exportKey('raw', key));

        // submit participation to server
        const res = await axios.post(`/api/participations`, {
          iv: toHex(iv),
          data: toHex(data),
          key: p.rewardKey,
        });

        const url = `${res.data.url}/#${participationKey}`;
        const participationCode = await QRCode.toDataURL(url);

        const canvas = document.createElement('canvas');
        canvas.height = 880;
        canvas.width = 600;
        
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = 'white';
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        ctx.font = 'bold 28px Roboto, sans-serif';
        ctx.fillStyle = 'black';
        ctx.fillText('PrePaMS Participation Code', 100, 50);

        const img = new Image();
        img.src = logo;
        await new Promise(resolve => img.addEventListener('load', resolve));
        ctx.drawImage(img, 2, 0, 90, 90);

        const qr = new Image();
        qr.src = participationCode;
        await new Promise(resolve => qr.addEventListener('load', resolve));
        ctx.drawImage(qr, 25, 115, 550, 550);

        ctx.font = '23px Roboto, sans-serif';
        [
          'This is your participation code for the study:',
          `> ${study.name}`,
          `organized by ${study.owner}.`,
          '',
          'The embedded information allows the study organizer',
          'to validate your participation and transfer participation',
          'rewards to you, but does not contain any personal',
          'identifiable information about you.',
        ].forEach((e, i) => ctx.fillText(e, 18, 710 + 22 * i))

        ctx.font = '16px Roboto, sans-serif';
        ctx.fillStyle = 'grey';
        ctx.fillText(study.id, 100, 78);

        ctx.textAlign = 'right';
        ctx.fillText(new Date().toJSON().slice(0, 10), 578, 58);
        ctx.fillText(new Date().toJSON().slice(11, 19), 578, 78);

        const participationFile = canvas.toDataURL();

        return {
          participationCode,
          participationFile,
          participationURL: url,
          participation: participation.serialize()
        };
      } catch {
        throw new Error('Prerequisites not met');
      }
    },

    async checkParticipation(context, { id, key }) {
      const res = await axios.get(`/api/participations/${id}`);

      const iv = fromHex(res.data.iv);
      const decryptionKey = await window.crypto.subtle.importKey('raw', fromHex(key), { name: 'AES-GCM' }, false, ['decrypt']);
      const data = await window.crypto.subtle.decrypt({ name: 'AES-GCM', iv }, decryptionKey, fromHex(res.data.data));

      const p = Participation.deserializeBinary(new Uint8Array(data));
      return {
        ...p.serialize(),
        valid: context.state.user.credential.checkParticipation(p)
      }
    },

    async rewardParticipation(context, participation) {
      const p = Participation.deserialize(participation);
      const reward = context.state.user.credential.issueReward(p);
      const res = await axios.post(`/api/rewards`, reward.serialize());
      return res.data;
    },

    async createStudy(context, study) {
      const { id } = new Resource(study.reward).serialize();
      const resource = [
        context.state.user.id,
        id,
        study.name,
        study.abstract,
        study.description,
        study.duration,
        study.reward,
        study.qualifier.sort().join(','),
        study.disqualifier.sort().join(','),
        study.webBased,
        study.studyUrl,
      ].join('|');

      const signature = context.state.user.credential.signResource(resource);

      const res = await axios.post('/api/studies', {
        ...study,
        id: id,
        signature: signature,
        owner: context.state.user.id
      });

      if (res.data.id) {
        return res.data.id
      }
    },

    async refreshBalance(context) {
      const res = await axios.get(`/api/rewards/`);

      const [balance, participated] = context.state.user.credential.getBalance(res.data.transactions, res.data.spend.map(e => e.tag));

      context.commit('newBalance', { balance, participated });
    },

    async payout(context, request) {
      const res = await axios.get(`/api/rewards/`);

      const [cost, req] = context.state.user.credential.requestPayout(
        request.amount,
        request.target,
        context.state.user.credential.identity,
        res.data.transactions,
        res.data.spend.map(e => e.tag)
      );

      if (!await request.source.$root.$confirm('Do you want to request the following payout?', `${request.amount} credits as ${request.target} for ${request.id}.${(cost - request.amount) > 0 ? ` ${cost - request.amount} additional credits will be lost during this transfer.` : ''}`)) {
        return;
      }

      const { data } = await axios.post(`/api/payout`, req);
      return data;
    }
  }
});
