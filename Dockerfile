# ==============================================================================
# Multi-stage Dockerfile per applicazione Dioxus 0.7 Fullstack (clord)
# Basato sulla guida ufficiale: https://dioxuslabs.com/learn/0.7/tutorial/deploy
# ==============================================================================

# Fase 1: Base chef con cargo-chef per il caching efficiente delle dipendenze
FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Fase 2: Planner che calcola la 'ricetta' delle dipendenze dal codice sorgente
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Fase 3: Builder che compila prima le dipendenze in cache, poi l'app Dioxus
FROM chef AS builder
# Installa il target WebAssembly necessario per la parte client di Dioxus
RUN rustup target add wasm32-unknown-unknown

# Compila solo le dipendenze (cache layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copia l'intero sorgente
COPY . .

# Installa la CLI di Dioxus (dx) tramite cargo-binstall per massima velocità
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

# Genera il bundle web di release (compila client WASM + server executable)
RUN dx bundle --web --release

# Fase 4: Runtime minimale Debian Bookworm Slim
FROM debian:bookworm-slim AS runtime

# Collega automaticamente l'immagine al repository Forgejo Packages
LABEL org.opencontainers.image.source="https://source.shadowfenix.net/development/clord"

# Installa i certificati CA necessari per eventuali chiamate HTTPS in uscita
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Crea la directory dell'applicazione e la cartella per i dati persistenti
WORKDIR /usr/local/app
RUN mkdir -p /usr/local/app/data

# Copia l'output del bundle prodotto da dx per il pacchetto "clord"
COPY --from=builder /app/target/dx/clord/release/web/ /usr/local/app

# Configurazione di rete predefinita del server Dioxus
ENV PORT=8080
ENV IP=0.0.0.0
EXPOSE 8080

ENTRYPOINT [ "/usr/local/app/server" ]
