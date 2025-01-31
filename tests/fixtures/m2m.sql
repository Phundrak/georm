INSERT INTO genres (name)
VALUES ('fantasy'),
       ('horror'),
       ('classic');

INSERT INTO book_genres (book_id, genre_id)
VALUES (1, 1),
       (1, 3),
       (2, 1),
       (2, 3),
       (3, 1),
       (3, 3),
       (4, 2),
       (4, 3);
