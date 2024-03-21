--------------------------------------------------------------------------------
-- Up
--------------------------------------------------------------------------------

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    role TEXT,
    publicKey BLOB
);

CREATE TABLE issued (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    signature BLOB
);

CREATE TABLE studies (
    id TEXT PRIMARY KEY,
    owner TEXT,
    name TEXT,
    abstract TEXT,
    description TEXT,
    duration TEXT,
    reward INTEGER,
    qualifier TEXT,
    disqualifier TEXT,
    constraints TEXT,
    webBased BOOLEAN,
    studyURL TEXT,
    signature BLOB,
    FOREIGN KEY(owner) REFERENCES users(id)
);

CREATE TABLE participations (
    id TEXT PRIMARY KEY,
    iv BLOB,
    data BLOB
);

CREATE TABLE ledger (
    id INTEGER PRIMARY KEY,
    participation TEXT,
    iv BLOB,
    data BLOB,
    tag TEXT,
    study TEXT,
    request BLOB,
    signature BLOB,
    value INTEGER,
    coin BLOB,
    chain BLOB,
    FOREIGN KEY(study) REFERENCES studies(id)
);

--------------------------------------------------------------------------------
-- Down
--------------------------------------------------------------------------------

DROP TABLE users;
DROP TABLE issued;
DROP TABLE studies;
DROP TABLE participations;
DROP TABLE rewards;
