--------------------------------------------------------------------------------
-- Up
--------------------------------------------------------------------------------

INSERT INTO users VALUES (
    'organizer@example.com',
    'organizer',
    '4acfd4LKlDx52P0ifxgnznplOHJutUXobPy7ism_xUI'
);

INSERT INTO studies VALUES (
    'CqDYCcS9vnZSKGDNIkEql1bfOBluvUvchrdgQ6Ijv0c',
    'organizer@example.com',
    'COVID-19, Personality and Social Media Usage',
    'Earn 5 easy credits by completing our questionnaire on social media use during the COVID-19 pandemic and personality traits.',
    '-',
    '30 minutes',
    5,
    '[]',
    '["MZQOjuGaFLfCMzVTuu2xYabC03kilWsRxPCPz9Rl7Cg"]',
    0,
    NULL
);

INSERT INTO studies VALUES (
    'Xp5UmTZd-1hyhvJ7Ct9hv1amLyhaJPBi8mdmvcZwGi8',
    'organizer@example.com',
    'COVID-19, Personality and Social Media Usage - Follow-Up',
    'Follow-up questionnaire on social media use during the continuing COVID-19 pandemic.',
    '-',
    '25 minutes',
    5,
    '["CqDYCcS9vnZSKGDNIkEql1bfOBluvUvchrdgQ6Ijv0c"]',
    '[]',
    0,
    NULL
);
INSERT INTO studies VALUES (
    'MZQOjuGaFLfCMzVTuu2xYabC03kilWsRxPCPz9Rl7Cg',
    'organizer@example.com',
    'Chatbot-based Training for Stress Reduction and Health Benefits',
    'The purpose of this study is to investigate the extent to which this chat-bot based training can reduce stress and improve health at the same time.',
    '-',
    '3 weeks',
    45,
    '[]',
    '[]',
    0,
    NULL
);

--------------------------------------------------------------------------------
-- Down
--------------------------------------------------------------------------------

DELETE FROM users WHERE id = 'organizer@example.com';
DELETE FROM studies WHERE owner = 'organizer@example.com';
