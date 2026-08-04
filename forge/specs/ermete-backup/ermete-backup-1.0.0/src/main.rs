use zbus::{interface, ConnectionBuilder, SignalContext};
use zbus::fdo::Result;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

struct BackupService;

#[interface(name = "org.ermete.Backup")]
impl BackupService {
    /// Esegue il backup chiamando borg create in background, emettendo segnali di progresso
    #[zbus(name = "PerformBackup")]
    async fn perform_backup(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>) -> Result<()> {
        let ctxt = ctxt.into_owned();

        // Spawn asincrono per l'esecuzione di borg
        tokio::spawn(async move {
            let _ = BackupService::backup_progress(&ctxt, "Avvio pipeline deduplicazione borg...".to_string()).await;
            
            let mut cmd = Command::new("borg");
            cmd.arg("create")
                .arg("--info")
                .arg("--progress")
                .arg("/var/backup/borg-repo::backup-{now}") // TODO: path reali
                .arg("/var/home/ermete") // TODO: path reali
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = BackupService::backup_progress(&ctxt, format!("Errore avvio borg: {}", e)).await;
                    return;
                }
            };

            // Catturiamo stderr per il progresso di borg
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = BackupService::backup_progress(&ctxt, line).await;
                }
            }
            
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = BackupService::backup_progress(&ctxt, line).await;
                }
            }

            match child.wait().await {
                Ok(status) if status.success() => {
                    let _ = BackupService::backup_progress(&ctxt, "Backup completato con successo.".to_string()).await;
                }
                Ok(status) => {
                    let _ = BackupService::backup_progress(&ctxt, format!("Backup terminato con errore: {}", status)).await;
                }
                Err(e) => {
                    let _ = BackupService::backup_progress(&ctxt, format!("Errore di attesa del processo borg: {}", e)).await;
                }
            }
        });

        Ok(())
    }

    /// Segnale DBus per il progresso del backup
    #[zbus(signal)]
    async fn backup_progress(ctxt: &SignalContext<'_>, log_line: String) -> Result<()>;
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let service = BackupService;

    // Crea la connessione e registra il servizio
    let _connection = ConnectionBuilder::session()?
        .name("org.ermete.Backup")?
        .serve_at("/org/ermete/Backup", service)?
        .build()
        .await?;

    println!("Servizio DBus org.ermete.Backup in ascolto...");

    // Loop infinito per mantenere vivo il demone
    std::future::pending::<()>().await;

    Ok(())
}
