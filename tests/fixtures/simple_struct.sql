INSERT INTO biographies (content)
VALUES ('Some text'),
       ('Some other text');

INSERT INTO authors (name, biography_id)
VALUES ('J.R.R. Tolkien', 2),
       ('George Orwell', NULL),
       ('Jack London', 1);
