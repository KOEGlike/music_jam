CREATE TABLE access_tokens  (
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  access_token varchar NOT NULL UNIQUE,
  expires_at BIGINT NOT NULL,
  scope varchar NOT NULL,
  refresh_token varchar NOT NULL
);

CREATE TABLE hosts (
  id char(24) NOT NULL UNIQUE PRIMARY KEY,
  access_token char(24) UNIQUE REFERENCES access_tokens (id) ON DELETE CASCADE
);

CREATE TABLE jams (
  id varchar(6) NOT NULL UNIQUE PRIMARY KEY,
  max_song_count smallint NOT NULL,
  host_id char(24) UNIQUE NOT NULL REFERENCES hosts (id) ON DELETE CASCADE,
  name varchar(30) NOT NULL
);

CREATE TABLE users (
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  jam_id char(6) NOT NULL REFERENCES jams (id) ON DELETE CASCADE,
  name varchar(50) NOT NULL,
  pfp_id char(24) UNIQUE NOT NULL
);

CREATE TABLE images (
  jam_id char(6) NOT NULL REFERENCES jams (id) ON DELETE CASCADE,
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  url varchar(255) NOT NULL,
  width bigint,
  height bigint
);

CREATE TABLE songs (
  user_id char(24) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  id varchar(22) UNIQUE PRIMARY KEY NOT NULL,
  name varchar(50) NOT NULL,
  album varchar(50) NOT NULL,
  duration int NOT NULL,
  image_id char(22) NOT NULL REFERENCES images (id)
);

CREATE TABLE artists (
  id varchar(22) UNIQUE PRIMARY KEY NOT NULL,
  song_id varchar(22) NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
  name varchar(50) NOT NULL
);



CREATE TABLE votes (
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  user_id char(24) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  song_id varchar(22) NOT NULL REFERENCES songs (id) ON DELETE CASCADE
);