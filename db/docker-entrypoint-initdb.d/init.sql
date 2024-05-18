CREATE DOMAIN uid varchar(32) NOT NULL;

CREATE TABLE "users" (
  "id" uid UNIQUE PRIMARY KEY,
  "jam_id" uid,
  "username" varchar(50) NOT NULL,
  "profile_picture" uid UNIQUE NOT NULL
);

CREATE TABLE "hosts" (
  "id" uid UNIQUE PRIMARY KEY,
  "access_token" uid UNIQUE
);

CREATE TABLE "jams" (
  "id" uid  UNIQUE PRIMARY KEY,
  "max_song_count" int NOT NULL,
  "host_id" uid NOT NULL
);

CREATE TABLE "selected_songs" (
  "user_id" varchar(32) NOT NULL,
  "id" uid UNIQUE PRIMARY KEY
);

CREATE TABLE "access_tokens" (
  "id" uid UNIQUE PRIMARY KEY,
  "access_token" varchar UNIQUE,
  "token_type" varchar NOT NULL,
  "scope" varchar NOT NULL,
  "expires_in" int NOT NULL,
  "refresh_token" varchar NOT NULL
);

ALTER TABLE "users" ADD FOREIGN KEY ("jam_id") REFERENCES "jams" ("id") ON DELETE CASCADE;

ALTER TABLE "selected_songs" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE;

ALTER TABLE "hosts" ADD FOREIGN KEY ("access_token") REFERENCES "access_tokens" ("id") ON DELETE CASCADE;

ALTER TABLE "jams" ADD FOREIGN KEY ("host_id") REFERENCES "hosts" ("id") ON DELETE CASCADE;

