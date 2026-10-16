# Development

Your new bare-bones project includes minimal organization with a single `main.rs` file and a few assets.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```



### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve --platform web
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

## Docker & Deployment

Il progetto include la configurazione Docker e Docker Compose ottimizzata secondo la [guida ufficiale Dioxus 0.7](https://dioxuslabs.com/learn/0.7/tutorial/deploy), utilizzando `cargo-chef` per il caching delle dipendenze.

### Avvio rapido con Docker Compose

Sul server o macchina remota con Docker:

```bash
docker compose up -d --build
```

L'app sarà raggiungibile all'indirizzo `http://<host-ip>:8080`.

I dati delle sessioni word cloud vengono salvati in `./data/wordclouds.json` e persistiti tramite il volume montato `./data`.

### Aggiornamento dell'applicazione

Per aggiornare l'applicazione dopo nuove modifiche al codice:

```bash
# Tramite lo script dedicato:
./deploy.sh

# Oppure manualmente:
git pull
docker compose up -d --build
```

### Comandi utili

```bash
# Visualizzare i log in tempo reale
docker compose logs -f

# Riavviare il servizio
docker compose restart

# Arrestare il container
docker compose down
```



