CREATE DOMAIN uid varchar(24) NOT NULL;

CREATE TYPE access_token_type AS (
  access_token varchar,
  expires_at BIGINT ,
  scope varchar ,
  refresh_token varchar,
);

CREATE TABLE "users" (
  "id" uid UNIQUE PRIMARY KEY,
  "jam_id" uid,
  "username" varchar(50) NOT NULL,
  "profile_picture" uid UNIQUE NOT NULL
);

CREATE TABLE "hosts" (
  "id" uid UNIQUE PRIMARY KEY,
  "access_token" access_token UNIQUE
);

CREATE TABLE "jams" (
  "id" varchar(6) NOT NULL UNIQUE PRIMARY KEY,
  "max_song_count" int NOT NULL,
  "host_id" uid UNIQUE NOT NULL,
  "name" varchar(30) NOT NULL,
);

CREATE TABLE "selected_songs" (
  "user_id" varchar(32) NOT NULL,
  "id" uid UNIQUE PRIMARY KEY
);





ALTER TABLE "users" ADD FOREIGN KEY ("jam_id") REFERENCES "jams" ("id") ON DELETE CASCADE;

ALTER TABLE "selected_songs" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE;


ALTER TABLE "jams" ADD FOREIGN KEY ("host_id") REFERENCES "hosts" ("id") ON DELETE CASCADE;

