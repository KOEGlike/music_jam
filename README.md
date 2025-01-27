# Music Jam

[![Docker](https://github.com/KOEGlike/music_jam/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/KOEGlike/music_jam/actions/workflows/docker-publish.yml)

This web app is for solving the problem of bad music at parties.
The users can submit songs to a queue and like those songs, the song with the most likes gets played using a Spotify integration.

## Features ✨

- Realtime ui with WebSockets
- Clean glassmorphic UI
- Spotify integration
- Quick joining, with QR code, and PFPs
- Rust

## Tech Stack

- Rust fronted, with Leptos(WASM)
- Rust backend
- Postgresql DB
- SCSS styling

## Gallery

![screenshot of host page](https://github.com/KOEGlike/music_jam/blob/main/images%2Fhost.png)
![screenshot of user page](https://github.com/KOEGlike/music_jam/blob/main/images%2Fuser.png)
![screenshot of create user page](https://github.com/KOEGlike/music_jam/blob/main/images%2Fcreate-user.png)

###

## Running the app locally

There are two ways to run the app, in a docker container, or on you machine directly.

I would recommend the containered version if you are not planning on developing the app, because it's a lot easier to do, and there are no advantages as a user to not run in a container.

It's recommended to install docker for both options, since it's way easier to run the db in a container than on you local machine.

### For both options you need to do these steps

1. Install Docker, see tutorial on dockers website
2. Clone this repo
3. Create a `.env` file in the root directory
4. Paste the contents of `.env.sample` into the newly created file
5. Fill it out with your own information
    1. `SPOTIFY_SECRET` and `SPOTIFY_ID` you can find these by creating spotify developer account, than crating a new app
    2. `POSTGRES_PASSWORD` env is the password of the database that the app uses, changed this to a secure password, realistically you will never interact with the DB, but it's good practice to have a secure password
    3. `SITE_URL` the url where the site will be deployed, for example `localhost:3000`, this is needed for the spotify oauth, make sure that you added this url in the spotify dashboard of your app as a redirect url
    4. `DATABASE_URL` the url of your database, you don't need this if you are using the container, usually `localhost`

### For the containered version

1. The app runs on localhost:8080 by default, you can change this by going into the `compose.yml` file and enditing the exposed port under the ports section of the app service. If you are confused ask chatgpt.
2. Run `docker compose up -d`
3. And boom, the app should run on the port that you selected
4. If it doesn't work, run `docker compose up`, so you can see where did things go wrong

### For the local version

1. Install rust nightly, by running: `rustup install nightly`
2. Install `sqlx-cli`, by running: `cargo install sqlx-cli`
3. Install `cargo-leptos`, by running: `cargo install cargo-leptos`
4. Start up the db, by running: `docker compose up jam-db -d`
5. Run the migrations on the db (make sure you set the `DATABASE_URL` env, in `.env`), by running: `sqlx database reset --source ./db/migrations`
6. To start the app run: `cargo leptos serve`
