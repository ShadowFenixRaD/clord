#!/bin/sh
set -e

echo "=== [clord] Aggiornamento e avvio container ==="

# Se il repository è gestito con git, scarica le ultime modifiche
if [ -d .git ]; then
    echo "-> Scaricamento aggiornamenti dal repository git..."
    git pull
fi

# Ricompila l'immagine con Docker BuildKit / cache e riavvia i container
echo "-> Costruzione e avvio con Docker Compose..."
docker compose up -d --build

# Pulizia di immagini intermedie non più utilizzate
echo "-> Pulizia immagini dangling..."
docker image prune -f

echo "=== [clord] Operazione completata con successo! ==="
docker compose ps
