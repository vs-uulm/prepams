--------------------------------------------------------------------------------
-- Up
--------------------------------------------------------------------------------

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    role TEXT,
    publicKey TEXT
);

CREATE TABLE issued (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    signature TEXT
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
    webBased BOOLEAN,
    studyURL TEXT,
    FOREIGN KEY(owner) REFERENCES users(id)
);

CREATE TABLE participations (
    id TEXT PRIMARY KEY,
    pk TEXT,
    iv TEXT,
    data TEXT
);

CREATE TABLE rewards (
    pk TEXT PRIMARY KEY,
    id TEXT,
    tag TEXT,
    data TEXT,
    value INTEGER,
    FOREIGN KEY(id) REFERENCES studies(id)
);

CREATE TABLE spend (
    tag TEXT PRIMARY KEY
);

CREATE TABLE payouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient TEXT,
    value INTEGER,
    target TEXT,
    receipt TEXT,
    FOREIGN KEY(recipient) REFERENCES studies(recipient)
);

--------------------------------------------------------------------------------
-- Down
--------------------------------------------------------------------------------

DROP TABLE users;
DROP TABLE issued;
DROP TABLE studies;
DROP TABLE participations;
DROP TABLE rewards;
DROP TABLE spend;
DROP TABLE payouts;
