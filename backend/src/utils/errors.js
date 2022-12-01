class HTTPError extends Error {
  constructor(message = undefined, code = 500) {
    super(message);
    this.status = code;
  }
};

class BadRequest extends HTTPError {
  constructor(message = 'Bad Request') {
    super(message, 400);
  }
};

class NotFound extends HTTPError {
  constructor() {
    super('Not Found', 404);
  }
};

module.exports = {
  HTTPError,
  NotFound,
  BadRequest
};
