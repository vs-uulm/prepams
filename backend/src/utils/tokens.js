const jwt = require('jsonwebtoken');
const crypto = require('crypto');

const ALGO = 'aes-256-gcm';

module.exports = {
  create(payload) {
    return new Promise((resolve, reject) => {
      const iv = crypto.randomBytes(12);
      const key = Buffer.from(process.env['TOKEN_ENCRYPTION_SECRET'], 'hex');
      const cipher = crypto.createCipheriv(ALGO, key, iv);
  
      let enc = cipher.update(JSON.stringify(payload), 'utf8', 'base64');
      enc += cipher.final('base64');
      const data = {
        e: enc,
        i: iv.toString('base64'),
        a: cipher.getAuthTag().toString('base64')
      };

      jwt.sign(data, process.env['TOKEN_SIGNING_SECRET'], { expiresIn: '120h' }, (err, token) => {
        if (err) {
          return reject(err);
        }

        resolve(token);
      });
    });
  },

  validate(token) {
    return new Promise((resolve, reject) => {
      if (!token) {
        return reject(new Error('no token supplied'));
      }

      jwt.verify(token, process.env['TOKEN_SIGNING_SECRET'], function(err, decoded) {
        if (err) {
          return reject(err);
        }

        const iv = Buffer.from(decoded.i, 'base64');
        const key = Buffer.from(process.env['TOKEN_ENCRYPTION_SECRET'], 'hex');
        const decipher = crypto.createDecipheriv(ALGO, key, iv);

        decipher.setAuthTag(Buffer.from(decoded.a, 'base64'));
        let payload = decipher.update(decoded.e, 'base64', 'utf8');
        payload += decipher.final('utf8');

        return resolve(JSON.parse(payload));
      });
    });
  }
};
