import Vue from 'vue';
import Vuex from 'vuex';

import axios from 'axios';
import QRCode from 'qrcode';

import logo from '@/assets/logo.png';

import { init, Organizer, Participant, Participation, Resource } from 'prepams-shared';
init();

const worker = new Worker(new URL('./worker.js', import.meta.url));

let workerPromises = {};

worker.addEventListener('message', ({ data }) => {
  if (workerPromises[data.id]) {
    if (data.error) {
      workerPromises[data.id].reject(data.result);
    } else {
      workerPromises[data.id].resolve(data.result);
    }
    delete workerPromises[data.id];
  }
});

function callWorker(args) {
  const id = `call-${(Math.random() + 1).toString(36)}`;
  return new Promise((resolve, reject) => {
    workerPromises[id] = { resolve, reject };
    worker.postMessage({ id, args });
  });
}

Vue.use(Vuex);

const encoder = new TextEncoder();
const toHex = a => new Uint8Array(a).reduce((s, b) => s + b.toString(16).padStart(2, '0'), '');
const fromHex = s => new Uint8Array(s.match(/.{1,2}/g).map(b => parseInt(b, 16)));

async function base64Decode(data) {
  const res = await fetch(`data:application/octet-stream;base64,${data}`);
  return new Uint8Array(await res.arrayBuffer());
}
async function base64Encode(bytes) {
  const res = await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(new File([bytes], '', { type: 'application/octet-stream' }));
  });
  return res.slice(res.indexOf(',') + 1);
}

const keys = {};
(async () => {
  try {
    if (!localStorage.getItem('keys')) {
      // auto login for demo accounts
      const req = await axios.get('/api/demo/credentials');
      localStorage.setItem('keys', JSON.stringify(Object.fromEntries(req.data.map(e => [e.id, e.key]))));

      for (const credential of req.data) {
        const cred = (credential.role === 'participant' ? Participant : Organizer).deserialize(fromHex(credential.state));
        keys[credential.id] = await window.crypto.subtle.importKey('jwk', credential.key, { name: 'AES-GCM' }, true, [ 'encrypt', 'decrypt' ])
        await persistCredential(cred);
      }
      if (req.data[0]) {
        localStorage.setItem('credential', req.data[0].id);
      }
    } else {
      for (const [id, key] of Object.entries(JSON.parse(localStorage.getItem('keys')))) {
        keys[id] = await window.crypto.subtle.importKey('jwk', key, { name: 'AES-GCM' }, true, [ 'encrypt', 'decrypt' ])
      }
    }
  } catch {
    // ignore
  }
})();

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

  const data = credential.serialize();
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
        return Participant.deserialize(new Uint8Array(plaintext));
      case 'organizer':
        return Organizer.deserialize(new Uint8Array(plaintext));
      default:
        throw new Error(`${role} not supported`);
    }
  } catch (e) {
    console.log(e);
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
        user.participated = new Set();
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
        await context.commit('authenticated', {
          balance: 0,
          id: credential.id,
          role: credential.role,
          attributes: role === 'participant' ? credential.attributes : null,
          credential: credential
        });
      } catch (ex) {
        console.log(ex);
        const e = new Error('incorrect password');
        e.role = role;
        e.id = id;
        throw e;
      }
    },

    async recover(context, data) {
      for (const role of [Participant, Organizer]) {
        try {
          const credential = role.deserialize(data);
          await context.commit('authenticated', {
            balance: 0,
            id: credential.id,
            role: credential.role,
            attributes: credential.role === 'participant' ? credential.attributes : null,
            credential: credential
          });
          return;
        } catch {
          // ignore
        }
      }

      throw new Error('could not recover account from code');
    },

    async signin(context, { id, password, role, attributes }) {
      if (localStorage.getItem(`${role}:${id}`)) {
        await deriveKeys(id, password);
        await context.dispatch('switchAccount', id);
      } else if (role === 'participant') {
        // try to recover account from public issued log
        const seed = await deriveKeys(id, password);
        const req = await axios.get(`/api/auth/signin?role=${role}`);
        const participant = new Participant(id, new Uint32Array(attributes.map(e => Number(e))), await base64Decode(req.data.issuer.lvk));

        participant.requestCredential(
          await base64Decode(req.data.issuer.pk),
          await base64Decode(req.data.issuer.vk),
          seed
        );

        for (const signature of req.data.log) {
          try {
            await participant.retrieveCredential(await base64Decode(signature));
            await context.commit('authenticated', {
              balance: 0,
              id: id,
              role: role,
              attributes: participant.attributes,
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
        const user = new Organizer(id, await base64Decode(req.data.issuer.pk), seed);

        const pk = await base64Encode(user.publicKey);

        if (!req.data.publicKeys.includes(pk)) {
          throw new Error('username or password invalid');
        }

        await context.commit('authenticated', {
          balance: 0,
          id: id,
          role: role,
          attributes: null,
          credential: user
        });
      }
    },

    async signup(context, { id, password, role, attributes }) {
      const seed = await deriveKeys(id, password);
      const [pk, vk, lvk] = await Promise.all([
        axios.get('/api/issuer/pk', { responseType: 'arraybuffer' }),
        axios.get('/api/issuer/vk', { responseType: 'arraybuffer' }),
        axios.get('/api/ledger/vk', { responseType: 'arraybuffer' })
      ]);

      let user = null;
      let request = null;

      if (role === 'participant') {
        user = new Participant(id, new Uint32Array(attributes.map(e => Number(e))), new Uint8Array(lvk.data));
        request = user.requestCredential(
          new Uint8Array(pk.data),
          new Uint8Array(vk.data),
          seed
        );
      } else {
        user = new Organizer(id, new Uint8Array(pk.data), seed);
        request = user.publicKey;
      }

      const res = await axios.post(`/api/auth/signup`, request, {
        headers: { 'Content-Type': 'application/octet-stream' },
        responseType: 'arraybuffer',
        params: { id, role }
      });

      if (role === 'participant') {
        user.retrieveCredential(new Uint8Array(res.data));
      }

      const state = user.serialize();
      const recoveryCode = await QRCode.toDataURL([{ data: state, mode: 'byte' }], {
        errorCorrectionLevel: 'L'
      });

      const canvas = document.createElement('canvas');
      canvas.height = 1120;
      canvas.width = 768;
      
      const ctx = canvas.getContext('2d');
      ctx.fillStyle = 'white';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      ctx.font = 'bold 28px Roboto, sans-serif';
      ctx.fillStyle = 'black';
      ctx.fillText('PrePaMS Restore Code', 132, 50);

      ctx.font = '27px Roboto, sans-serif';
      ctx.textAlign = 'left';
      ctx.fillText(user.identity, 132, 88);

      const img = new Image();
      img.src = logo;
      await new Promise(resolve => img.addEventListener('load', resolve));
      ctx.drawImage(img, 32, 10, 90, 90);

      const qr = new Image();
      qr.src = recoveryCode;
      await new Promise(resolve => qr.addEventListener('load', resolve));
      ctx.drawImage(qr, 32, 125, 708, 708);

      [
        'This QR code is your recovery key.  If you lose access',
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
      ].forEach((e, i) => ctx.fillText(e, 48, 875 + 22 * i))

      ctx.font = '22px Roboto, sans-serif';
      ctx.textAlign = 'right';
      ctx.fillStyle = 'grey';
      ctx.fillText(new Date().toJSON().slice(0, 10), 726, 50);
      ctx.fillText(new Date().toJSON().slice(11, 19), 726, 86);

      const recoveryFile = canvas.toDataURL();

      await context.commit('authenticated', {
        balance: 0,
        id: id,
        role: role,
        credential: user,
        attributes: role === 'participant' ? user.attributes : null,
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

        let encoded = await callWorker({
          call: 'participate',
          credential: context.state.user.credential.serialize(),
          resource: {
            ...study,
            qualifier: study.qualifier.map(id => [
              id,
              participations.filter(e => e.study === id).map(e => e.tag)
            ]),
            disqualifier: study.disqualifier.map(id => [
              id,
              participations.filter(e => e.study === id).map(e => e.tag)
            ])
          }
        });

        const iv = window.crypto.getRandomValues(new Uint8Array(12));
        const key = await window.crypto.subtle.generateKey({
          name: 'AES-GCM',
          length: 256
        }, true, ['encrypt']);

        const data = new Uint8Array(await window.crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, encoded));

        const participationKey = toHex(await window.crypto.subtle.exportKey('raw', key));

        const request = new Uint8Array(data.length + 12);
        request.set(data, 12);
        request.set(iv);

        let participationCode = null;
        let participationFile = null;

        // submit participation to server
        const res = await axios.post(`/api/participations`, request, {
          headers: { 'Content-Type': 'application/octet-stream' },
        });

        const url = `${res.data.url}/#${participationKey}`;

        if (!study.webBased) {
          participationCode = await QRCode.toDataURL(url);

          const wrappedName = study.name.split(' ').reduce((s, e) => {
            if (s[s.length - 1].length + e.length > 42) {
              if (e.length + 3 > 42) {
                // break anywhere
                let offset = 0;

                if (s[s.length - 1].length < 30) {
                  offset = 36 - s[s.length - 1].length;
                  s[s.length - 1] = `${s[s.length - 1]} ${e.slice(0, offset)}`;
                }

                while (offset < e.length) {
                  const slice = e.slice(offset, offset + 34);
                  offset += slice.length;
                  if (!slice.length) {
                    break;
                  }
                  s.push(` > ${slice}`);
                }
              } else {
                s.push(` > ${e}`);
              }
            } else {
              s[s.length - 1] = `${s[s.length - 1]} ${e}`;
            }
            return s;
          }, [' >']);

          const canvas = document.createElement('canvas');
          canvas.height = 870 + 22 * wrappedName.length;
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
            ...wrappedName,
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

          participationFile = canvas.toDataURL();
        } else {
          encoded = await base64Encode(encoded);
          participationCode = res.data.id;
        }

        return {
          participationCode,
          participationFile,
          participationURL: url,
          participation: encoded
        };
      } catch (e) {
        console.log(e);
        throw new Error('Prerequisites not met');
      }
    },

    async checkParticipation(context, { id, key }) {
      const res = await axios.get(`/api/participations/${id}`, { responseType: 'arraybuffer' });

      const buffer = new Uint8Array(res.data);
      const iv = buffer.slice(0, 12);

      const decryptionKey = await window.crypto.subtle.importKey('raw', fromHex(key), { name: 'AES-GCM' }, false, ['decrypt']);
      const data = await window.crypto.subtle.decrypt({ name: 'AES-GCM', iv }, decryptionKey, buffer.slice(12));

      return await callWorker({
        call: 'verify',
        id: id,
        participation: new Uint8Array(data),
        rewarded: !!res.headers['x-rewarded']
      });
    },

    async rewardParticipation(context, participation) {
      const confirmedParticipation = context.state.user.credential.confirmParticipation(
        Participation.deserialize(new Uint8Array(participation.data)),
        participation.id
      );
      const res = await axios.post(`/api/rewards`, confirmedParticipation, {
        headers: { 'Content-Type': 'application/octet-stream' },
      });
      return res.data;
    },

    async createStudy(context, study) {
      const resource = new Resource(
        null,
        study.name || '',
        study.abstract || '',
        study.description || '',
        study.duration || '',
        study.reward,
        study.webBased,
        study.studyURL || '',
        study.qualifier,
        study.disqualifier,
        study.constraints 
      );

      const signature = context.state.user.credential.signResource(resource);

      const res = await axios.post('/api/studies', signature, {
        headers: { 'Content-Type': 'application/octet-stream' }
      });

      if (res.data.id) {
        return res.data.id
      }
    },

    async refreshBalance(context) {
      const res = await axios.get(`/api/ledger`, { responseType: 'arraybuffer' });
      const ledger = new Uint8Array(res.data);
      const [balance, participated] = context.state.user.credential.getBalance(ledger);

      context.commit('newBalance', {
        participated: new Set(participated),
        balance: balance
      });
    },

    async payout(context, request) {
      const res = await axios.get(`/api/ledger`, { responseType: 'arraybuffer' });
      const ledger = new Uint8Array(res.data);

      const nullRequest = context.state.user.credential.requestNulls();
      const nullResponse = await axios.post('/api/nulls', nullRequest.request(), {
        headers: { 'Content-Type': 'application/octet-stream' },
        responseType: 'arraybuffer'
      });

      const nulls = nullRequest.unblind(new Uint8Array(nullResponse.data));

      const { costs, proof } = await callWorker({
        call: 'payout',
        credential: context.state.user.credential.serialize(),
        recipient: context.state.user.id,
        amount: request.amount,
        target: request.target,
        ledger: ledger,
        nulls: nulls
      });

      if (!await request.source.$root.$confirm('Do you want to request the following payout?', `${request.amount} credits as ${request.target} for ${request.id}.${(costs - request.amount) > 0 ? ` ${costs - request.amount} additional credits will be lost during this transfer.` : ''}`)) {
        return;
      }

      const { data } = await axios.post(`/api/payout`, proof, {
        headers: { 'Content-Type': 'application/octet-stream' }
      });
      return data;
    }
  }
});
