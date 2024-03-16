# How to setup your system to contribute to Eventageous

- Install NPM version documented in the `.tool-versions` file (as of this writing, 20)
  - You can use [asdf-vm](https://asdf-vm.com/) if that's your thing
- Install [ember CLI](https://cli.emberjs.com/release/): `npm install -g ember`
- Install [shuttle CLI](https://docs.shuttle.rs/getting-started/installation#from-source): `cargo install cargo-shuttle`
- Build ember front-end by running these commands from the `frontend-ember` directory:
  - `npm install` -- first time only
  - `ember build` -- after making changes
- Setup [secrets](#secrets)
- Run a local instance by doing `cargo shuttle run` from the main project directory

## Secrets

Eventageous requires access to a Google calendar that is used as the backend. This is configuring using [Shuttle secrets](https://docs.shuttle.rs/resources/shuttle-secrets). You can configure the calendar via a `Secrets.toml` (or `Secrets.dev.toml`) like:

```
GOOGLE_API_KEY = "XXXX"
GOOGLE_CALENDAR_ID = "YYY"
```

Create an API key in your Google Cloud application, follow the _Credentials_ menu option and then _+ Create Credentials_ for _API Key_. Figuring out the details of how you want to set up your permissions and all the rest is an exercise to the reader.
