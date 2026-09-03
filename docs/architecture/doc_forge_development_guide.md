# Ermete Forge: Guida allo Sviluppo e all'Integrazione Pacchetti

Questa guida è destinata a ingegneri, sviluppatori e Agenti IA che necessitano di integrare nuovi pacchetti, demoni o configurazioni all'interno di Ermete OS.

In Ermete OS, l'aggiunta di un software **non avviene mai a runtime** (es. scaricando un eseguibile). Tutto deve passare attraverso l'infrastruttura **Ermete Forge**, che gestisce la compilazione distribuita, l'hashing e l'assemblaggio OCI (bootc/OSTree) in modo immutabile e Zero-Trust.

---

## 1. Architettura della Forge e DAG Orchestrator

La compilazione non è sequenziale. È orchestrata dal **DAG Orchestrator** (`forge/scripts/dag_orchestrator.py`), che analizza le dipendenze dei file `.spec` e le esegue in parallelo.
Per scalare su build-farm, l'orchestratore utilizza **Redis** (o un fallback locale su disco) per memorizzare in cache gli hash dei layer OCI e saltare le ricompilazioni inutili.

### Flusso di Aggiunta di un Pacchetto:
1. **Creazione dello Spec:** Creare la cartella `forge/specs/<nome-pacchetto>/` contenente il file `.spec` (standard RPM) e l'eventuale cartella `SOURCES/`. I file di supporto tracciati (unità systemd, configurazioni) vanno in `SOURCES/` e si dichiarano come `SourceN`; i tarball upstream **non** si committano (`*.gz` è ignorato): la spec dichiara l'URL e `SOURCES/sources.sha256` ne fissa il checksum.
2. **Definizione nel Manifest:** Aggiungere il pacchetto al manifest JSON (se applicabile, in `forge/config/packages.json`).
3. **Trigger della Pipeline:** L'orchestrazione (innescata da GitHub Actions o in locale tramite `./forge/scripts/build-offline.sh`) valuterà il DAG, capirà dove si posiziona il tuo pacchetto, spawnerà un micro-container OCI (Podman/Buildah), compilerà l'RPM ed esporterà l'artefatto in `~/.rpmbuild/RPMS/`.

---

## 2. Le 4 Regole d'Oro Zero-Trust (Pena: Fallimento della Build)

Per mantenere l'integrità del sistema, ogni `.spec` o `Containerfile` deve sottostare a regole brutali. Qualsiasi violazione verrà bloccata dall'Inquisitore CI (Vitreol).

### ❌ Regola 1: Divieto di Download Dinamici (No `curl | sh`)
Non è MAI permesso eseguire `curl`, `wget` o `git clone` all'interno delle fasi `%prep`, `%build` o `%install` del `.spec`, né disabilitare la verifica SSL (`http.sslVerify=false`).
**Soluzione:** Tutti i sorgenti devono essere dichiarati come `Source0`, `Source1`, ecc. nell'intestazione del `.spec`, con l'URL upstream (frammento `#/nome.tar.gz` in stile Fedora se il nome dell'archivio differisce). Prima di `rpmbuild`, `forge/scripts/fetch_sources.sh` scarica ciò che manca e verifica ogni file contro `SOURCES/sources.sha256` (formato di `sha256sum`): una voce assente nel manifest, un checksum diverso o un download fallito fermano la build.

### ❌ Regola 2: Nessuna Mutazione di `/usr` o `/etc` a Runtime (`%post`)
Non è permesso utilizzare lo scriptlet `%post` o script di provisioning eseguiti al boot per fare copie `cp` o `chmod` in `/usr` o in `/etc`. Questo distrugge l'immutabilità OCI (OSTree).
**Soluzione:** Ogni file di configurazione, binario o demone systemd DEVE essere copiato nei percorsi giusti *esclusivamente* durante la fase `%install` dello `.spec`. Per file dinamici, usare `systemd-tmpfiles`.

### ❌ Regola 3: Zero Disabilitazione di Sicurezza
Vietato impostare `repo_gpgcheck=0`. Vietato usare la macro `%undefine _fortify_source`. Ermete OS compila tutto con flag di hardenizzazione estremi (LTO, -O3, ASLR, CFI). Disabilitarli è vietato.

### ❌ Regola 4: Nessun Accesso di Rete in Compilazione
Il container OCI in cui avviene il `rpmbuild` viene istanziato con `bwrap --unshare-net` (o equivalenti podman). Qualsiasi pacchetto linguistico (NPM, Cargo, Pip) che tenta di scaricare dipendenze durante il `%build` fallirà.
**Soluzione:** Usare tool di *vendoring* (es. `cargo vendor`, `npm shrinkwrap`) o dichiarare i registry offline nei sorgenti `SourceX`.

---

## 3. Aggiungere un nuovo Demone Rust (Es. `ermete-example-daemon`)

Se sei un agente incaricato di scrivere un nuovo demone per l'OS, segui questa checklist:
1. Sviluppa il codice in `system/ermete-example-daemon/` assicurandoti di non usare blocchi `unsafe` senza estrema giustificazione.
2. Crea `forge/specs/ermete-example-daemon/ermete-example-daemon.spec`.
3. Nel `.spec`, definisci il pacchetto **senza** `Source0`: il crate vive nel workspace e il DAG compila il checkout in place (`rpmbuild --build-in-place`, scelto automaticamente per le spec senza `Source`), con la radice del repo come directory corrente di `%build` e `%install`. Le spec che dichiarano `Source` seguono invece il percorso ordinario, `%prep` incluso.
4. In `%build`, `cargo build --release --locked -p <nome-crate>`; i file di dati del crate si riferiscono tramite `%global crate_dir <percorso del crate dalla radice>`.
5. In `%install`, installa l'eseguibile in `/usr/libexec/` o `/usr/bin/` e il file `ermete-example-daemon.service` in `/usr/lib/systemd/system/`.
6. Nel `%post`, esegui solo `systemctl preset ermete-example-daemon.service` (mai systemctl start).

---

## 4. Test Locali prima del Commit

Prima di inviare una modifica a Ermete, un agente o sviluppatore deve validare:
```bash
# 1. Verifica statica della memoria (Agente Rust Paranoia)
cargo clippy --workspace --all-features -- -D warnings
cargo kani

# 2. Compilazione OCI isolata per testare il pacchetto
./forge/scripts/build-offline.sh <nome-pacchetto>
```
Se il build passa in locale senza richiedere rete, è pronto per essere assorbito dal Kernel Immutabile.
