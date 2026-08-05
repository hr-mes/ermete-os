use std::error::Error;
use futures_util::stream::StreamExt;
use tokio::process::Command;
use zbus::{Connection, MessageStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Si connette al bus di sistema
    let connection = Connection::system().await?;
    println!("ermete-recovery daemon avviato. In ascolto del segnale CriticalFailure su DBus...");

    // Aggiungiamo una regola di match per il segnale CriticalFailure
    let proxy = zbus::fdo::DBusProxy::new(&connection).await?;
    proxy
        .add_match_rule(
            "type='signal',interface='os.ermete.Recovery',member='CriticalFailure'"
                .try_into()?,
        )
        .await?;

    // Creiamo uno stream per ricevere i messaggi
    let mut stream = MessageStream::from(connection.clone());

    // Loop non bloccante per ascoltare i messaggi DBus
    while let Some(msg_result) = stream.next().await {
        match msg_result {
            Ok(msg) => {
                let header = msg.header();
                if header.message_type() == zbus::message::Type::Signal {
                    if let (Some(interface), Some(member)) = (header.interface(), header.member()) {
                        if interface.as_str() == "os.ermete.Recovery"
                            && member.as_str() == "CriticalFailure"
                        {
                            println!("Ricevuto segnale CriticalFailure. Avvio orchestratore di rollback asincrono...");
                            
                            // Cloniamo la connessione DBus da passare al task asincrono
                            let conn = connection.clone();
                            
                            // Eseguiamo il rollback in un task asincrono separato per non bloccare il loop
                            tokio::spawn(async move {
                                if let Err(e) = execute_rollback(&conn).await {
                                    eprintln!("Errore critico durante il rollback: {}", e);
                                }
                            });
                        }
                    }
                }
            }
            Err(e) => eprintln!("Errore nella lettura del messaggio DBus: {}", e),
        }
    }

    Ok(())
}

async fn execute_rollback(
    connection: &Connection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Tentativo di rollback tramite OSTree...");
    
    // Proviamo rpm-ostree rollback
    let ostree_output = Command::new("rpm-ostree")
        .arg("rollback")
        .output()
        .await?;

    if ostree_output.status.success() {
        println!("Rollback OSTree armato con successo. Al prossimo avvio verrà caricato il deployment precedente.");
        send_success_signal(connection).await?;
        return Ok(());
    }

    eprintln!("Rollback OSTree fallito. Tento fallback su BTRFS subvolume snapshot...");

    // Fallback btrfs snapshot
    let btrfs_output = Command::new("btrfs")
        .arg("subvolume")
        .arg("snapshot")
        .arg("/")
        .arg("/.recovery-snapshot-rollback")
        .output()
        .await?;

    if btrfs_output.status.success() {
        println!("Snapshot BTRFS creato con successo.");
        send_success_signal(connection).await?;
        return Ok(());
    }

    Err("Nessun metodo di rollback applicabile ha avuto successo".into())
}

async fn send_success_signal(connection: &Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Invia un segnale di success sul bus di sistema
    connection
        .emit_signal(
            None::<()>,
            "/os/ermete/Recovery",
            "os.ermete.Recovery",
            "RollbackArmed",
            &(),
        )
        .await?;
        
    println!("Segnale RollbackArmed emesso con successo.");
    Ok(())
}
