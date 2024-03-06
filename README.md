# Americano

Experimental calendaring software 🤘

## Setup required

- Install [shuttle CLI](https://docs.shuttle.rs/getting-started/installation#from-source): `cargo install cargo-shuttle`

### To work on the front-end

The front-end is based on ember. If you are making changes there you will need to install npm and the ember CLI:

- Install npm
- Install [ember CLI](https://cli.emberjs.com/release/): `npm install -g ember`

## Secrets

Americano requires access to a Google calendar that is used as the backend. This is configuring using [Shuttle secrets](https://docs.shuttle.rs/resources/shuttle-secrets). You can configure the calendar via a `Secrets.toml` (or `Secrets.dev.toml`) like:

```
GOOGLE_API_KEY = "XXXX"
GOOGLE_CALENDAR_ID = "YYY"
```

Getting the API key and calendar ID is left as an exercise to the reader (i.e., I forget how I did it now, but I remember it took more time to figure out than I expected).
