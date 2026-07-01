#!/bin/bash
set -e

echo "========================================================="
echo " ERMETE OS - BUILD LOCALE "
echo "========================================================="
echo "Questo script genera l'immagine OS finale usando Podman."
echo "Esegue il build dal Containerfile locale sfruttando i pacchetti OCI pubblicati."

podman build --security-opt label=disable --security-opt seccomp=unconfined -t localhost/ermete-os-local -f Containerfile .

echo "========================================================="
echo " Build Completata con Successo!"
echo " L'immagine è disponibile come localhost/ermete-os-local"
echo "========================================================="
