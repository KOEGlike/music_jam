CREATE TYPE access_token AS (
  access_token varchar,
  expires_at BIGINT,
  scope varchar,
  refresh_token varchar
);

CREATE TABLE hosts (
  id char(24) NOT NULL UNIQUE PRIMARY KEY,
  access_token access_token UNIQUE
);

CREATE TABLE jams (
  id varchar(6) NOT NULL UNIQUE PRIMARY KEY,
  max_song_count smallint NOT NULL,
  host_id char(24) UNIQUE NOT NULL REFERENCES hosts (id) ON DELETE CASCADE,
  name varchar(30) NOT NULL
);

CREATE TABLE users (
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  jam_id char(24) NOT NULL REFERENCES jams (id) ON DELETE CASCADE,
  name varchar(50) NOT NULL,
  pfp_id char(24) UNIQUE NOT NULL
);

CREATE TABLE songs (
  user_id char(24) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  id varchar(22) UNIQUE PRIMARY KEY
);