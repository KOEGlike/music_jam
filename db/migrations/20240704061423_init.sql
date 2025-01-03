

CREATE TABLE hosts (
  id char(24) NOT NULL UNIQUE PRIMARY KEY
);

CREATE TABLE access_tokens  (
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  access_token varchar NOT NULL UNIQUE,
  expires_at BIGINT NOT NULL,
  scope varchar NOT NULL,
  refresh_token varchar NOT NULL,
  host_id char(24) NOT NULL REFERENCES hosts (id) ON DELETE CASCADE
);

CREATE TABLE jams (
  id varchar(6) NOT NULL UNIQUE PRIMARY KEY,
  max_song_count smallint NOT NULL,
  host_id char(24) UNIQUE NOT NULL REFERENCES hosts (id) ON DELETE CASCADE,
  song_position real NOT NULL DEFAULT 0,
  name varchar(30) NOT NULL
);

CREATE TABLE users (
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  jam_id char(6) NOT NULL REFERENCES jams (id) ON DELETE CASCADE,
  name varchar(50) NOT NULL
);

CREATE TABLE songs (
  user_id char(24) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  spotify_id varchar  NOT NULL,
  name varchar NOT NULL,
  album varchar NOT NULL,
  duration int NOT NULL,
  artists varchar[] NOT NULL,
  image_url varchar NOT NULL
);




CREATE TABLE votes (
  id char(48) UNIQUE PRIMARY KEY NOT NULL,
  user_id char(24) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  song_id char(24) NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
  UNIQUE (song_id, user_id)
);
