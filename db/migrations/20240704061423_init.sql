CREATE TYPE access_token AS  (
  access_token varchar NOT N,
  expires_at BIGINT ,
  scope varchar ,
  refresh_token varchar
);

create access_token expires_at scope refresh_token as access_token 
check (
  (value).city is not null and 
  (value).address_line is not null and
  (value).zip_code is not null
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
  jam_id char(6) NOT NULL REFERENCES jams (id) ON DELETE CASCADE,
  name varchar(50) NOT NULL
);

CREATE TYPE sp_image AS(
  url varchar(255) ,
  width bigint,
  height bigint
);

CREATE TABLE songs (
  user_id char(24) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  id varchar(22) UNIQUE PRIMARY KEY NOT NULL,
  name varchar(50) NOT NULL,
  album varchar(50) NOT NULL,
  duration int NOT NULL,
  image sp_image NOT NULL,
  artists varchar(50)[] NOT NULL
);

CREATE TABLE votes (
  id char(24) UNIQUE PRIMARY KEY NOT NULL,
  user_id char(24) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  song_id varchar(22) NOT NULL REFERENCES songs (id) ON DELETE CASCADE
);
