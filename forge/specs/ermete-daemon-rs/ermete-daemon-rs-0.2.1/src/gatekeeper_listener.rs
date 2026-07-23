use zbus::{proxy, Connection};
use std::process::Command;

#[proxy(
    interface = "os.ermete.Gatekeeper",
    default_service = "os.ermete.Gatekeeper",
    default_path = "/os/ermete/Gatekeeper"
)]
trait Gatekeeper {
    #[zbus(signal)]
    fn prompt_required(&self, fd_id: u64, app_name: &str) -> zbus::Result<()>;

    fn approve_execution(&self, fd_id: u64) -> zbus::Result<()>;
    fn deny_execution(&self, fd_id: u64) -> zbus::Result<()>;
}

pub async fn start_gatekeeper_listener(sys_conn: Connection) -> zbus::Result<()> {
    let proxy = GatekeeperProxy::new(&sys_conn).await?;
    let mut stream = proxy.receive_prompt_required().await?;

    println!("Listening for Gatekeeper prompts on system bus...");

    tokio::spawn(async move {
        use futures_util::StreamExt;
        while let Some(signal) = stream.next().await {
            let args = signal.args().expect("Failed to parse signal args");
            let fd_id = args.fd_id;
            let app_name = args.app_name.to_string();
            let proxy_clone = proxy.clone();

            tokio::spawn(async move {
                println!("Received Gatekeeper prompt for {}. Showing UI...", app_name);
                
                // Spawn the shell to show the prompt
                let status = Command::new("ermete-shell-rs")
                    .arg("--gatekeeper-prompt")
                    .arg(&app_name)
                    .status();

                let approved = match status {
                    Ok(s) => s.success(),
                    Err(_) => false,
                };

                if approved {
                    println!("User approved execution for {}.", app_name);
                    let _ = proxy_clone.approve_execution(fd_id).await;
                } else {
                    println!("User denied execution for {}.", app_name);
                    let _ = proxy_clone.deny_execution(fd_id).await;
                }
            });
        }
    });

    Ok(())
}
