CREATE TABLE "users" (
  "id" integer UNIQUE PRIMARY KEY,
  "jam_id" integer NOT NULL,
  "username" varchar NOT NULL,
  "profile_picture" image
);

CREATE TYPE "access_token" (
  "access_token" varchar UNIQUE PRIMARY KEY,
  "token_type" varchar NOT NULL,
  "scope" varchar NOT NULL,
  "expires_in" int NOT NULL,
  "refresh_token" varchar NOT NULL
);

CREATE TABLE "jams" (
  "id" integer UNIQUE PRIMARY KEY NOT NULL,
  "max_song_count" int NOT NULL,
  "access_token" access_token UNIQUE NOT NULL
);

CREATE TABLE "selected_songs" (
  "user_id" integer NOT NULL,
  "id" varchar UNIQUE PRIMARY KEY NOT NULL,
  "album_cover" image NOT NULL
);



ALTER TABLE "jams" ADD FOREIGN KEY ("id") REFERENCES "users" ("jam_id") ON DELETE CASCADE;

ALTER TABLE "users" ADD FOREIGN KEY ("id") REFERENCES "selected_songs" ("user_id") ON DELETE CASCADE;g