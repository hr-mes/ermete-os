#!/bin/bash
# Chimera Auto-DMZ Fuzzer
# Esegue il build del Kernel. Se incontra un errore di compilazione (es. Werror, Objtool),
# ne estrae la directory, la declassa a O2 in tempo reale e rilancia la compilazione
# finché non ottiene un Kernel fuso al 100%.

MAX_RETRIES=20
RETRY_COUNT=0
REAL_MAKE="/usr/bin/make"

echo ">>> [AUTO-DMZ] Inizializzazione Holy Grail Fuzzer..."

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    echo ">>> [AUTO-DMZ] Lancio Kbuild (Tentativo $((RETRY_COUNT + 1))/$MAX_RETRIES)..."
    
    # Esegue il make reale in background/pipe per catturare lo stderr senza bloccare il TTY
    # Usa pipefail per catturare l'exit code di make
    set +e
    $REAL_MAKE "$@" 2> >(tee /tmp/chimera_err.log >&2)
    EXIT_CODE=$?
    set -e

    if [ $EXIT_CODE -eq 0 ]; then
        echo ">>> [AUTO-DMZ] Compilazione completata con successo! La Chimera è stabile."
        exit 0
    fi

    echo ">>> [AUTO-DMZ] Errore Fatale rilevato (Exit $EXIT_CODE). Analisi AST in corso..."
    
    # Estrae il file incriminato (il primo che ha scatenato un "error:")
    # Regex per catturare "path/to/file.c:line:col: error: "
    BAD_FILE=$(grep -m1 -E "error: " /tmp/chimera_err.log | awk -F':' '{print $1}')
    
    # Se il grep ha fallito, potrebbe essere un errore di Modpost o altro (non un "error:" di clang)
    if [ -z "$BAD_FILE" ] || [ ! -f "$BAD_FILE" ]; then
        echo ">>> [AUTO-DMZ] Impossibile determinare la cartella dall'errore o errore non sintattico. Aborto Fuzzer."
        exit $EXIT_CODE
    fi

    BAD_DIR=$(dirname "$BAD_FILE")
    echo ">>> [AUTO-DMZ] Infezione O3 isolata in: $BAD_DIR"
    
    # Risalita intelligente (DMZ Propagation)
    # Controlla se la stringa "ccflags-remove-y" è presente in Makefile o in Kbuild.
    # Se la cartella NON HA NÉ Makefile NÉ Kbuild, è una cartella ponte: la attraversiamo e continuiamo a risalire.
    # Se la cartella ha un Makefile che ha GIA' ricevuto l'infezione, risale.
    while { [ ! -f "$BAD_DIR/Makefile" ] && [ ! -f "$BAD_DIR/Kbuild" ]; } || { [ -f "$BAD_DIR/Makefile" ] && grep -q "ccflags-remove-y" "$BAD_DIR/Makefile" 2>/dev/null; } || { [ -f "$BAD_DIR/Kbuild" ] && grep -q "ccflags-remove-y" "$BAD_DIR/Kbuild" 2>/dev/null; }; do
        if [ ! -f "$BAD_DIR/Makefile" ] && [ ! -f "$BAD_DIR/Kbuild" ]; then
            echo ">>> [AUTO-DMZ] Cartella ponte, risalgo al livello superiore: $BAD_DIR"
        else
            echo ">>> [AUTO-DMZ] La cartella $BAD_DIR è già schermata. L'infezione è radicata nel modulo padre."
        fi
        BAD_DIR=$(dirname "$BAD_DIR")
        if [ "$BAD_DIR" == "." ] || [ "$BAD_DIR" == "drivers" ] || [ "$BAD_DIR" == "/" ]; then
            echo ">>> [AUTO-DMZ] Radice raggiunta. Errore insormontabile. Aborto Fuzzer."
            cat /tmp/chimera_err.log
            exit $EXIT_CODE
        fi
        echo ">>> [AUTO-DMZ] Risalgo al livello superiore: $BAD_DIR"
    done

    echo ">>> [AUTO-DMZ] Applicazione Scudo O2 in tempo reale su $BAD_DIR e suoi rami..."
    # Inietta lo scudo in tutti i Makefile e Kbuild figli
    find "$BAD_DIR" -type f \( -name "Makefile" -o -name "Kbuild" \) -exec bash -c 'cat << "EOF_MAKE" >> "$1"

ccflags-remove-y += -O3 $(CC_FLAGS_LTO)
ccflags-y += -O2 -fno-lto -Wno-error -Wno-unknown-warning-option
KCFLAGS := $(filter-out -O3,$(KCFLAGS))
EOF_MAKE
' _ {} \;
    
    RETRY_COUNT=$((RETRY_COUNT + 1))
done

echo ">>> [AUTO-DMZ] Raggiunto limite massimo di retry ($MAX_RETRIES). Il Kernel è troppo instabile."
exit 1
