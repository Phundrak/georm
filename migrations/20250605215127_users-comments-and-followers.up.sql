-- Add migration script here
CREATE TABLE Users (
  id SERIAL PRIMARY KEY,
  username VARCHAR(100) UNIQUE NOT NULL
);

CREATE TABLE Profiles (
  id SERIAL PRIMARY KEY,
  user_id INT UNIQUE NOT NULL,
  bio TEXT,
  display_name VARCHAR(100),
  FOREIGN KEY (user_id) REFERENCES Users(id)
);

CREATE TABLE Comments (
  id SERIAL PRIMARY KEY,
  author_id INT NOT NULL,
  content TEXT NOT NULL,
  FOREIGN KEY (author_id) REFERENCES Users(id)
);

CREATE TABLE Followers (
  id SERIAL PRIMARY KEY,
  followed INT NOT NULL,
  follower INT NOT NULL,
  FOREIGN KEY (followed) REFERENCES Users(id) ON DELETE CASCADE,
  FOREIGN KEY (follower) REFERENCES Users(id) ON DELETE CASCADE,
  CHECK (followed != follower),
  UNIQUE (followed, follower)
);
