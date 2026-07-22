use std::process::Command;

fn main() {
    println!("Starting Diagnostic: Network Stack...");
    let ping = Command::new("ping").args(["-c", "1", "1.1.1.1"]).output();
    
    match ping {
        Ok(out) if out.status.success() => println!("Network: OK"),
        Ok(_) => {
            println!("Network: FAILED. Reporting network failure gracefully to the UI...");
            glib::timeout_add_local_once(std::time::Duration::from_secs(1), || {
                println!("UI Network failure popup displayed.");
            });
        }
        Err(e) => eprintln!("Failed to execute ping: {}", e),
    }
}
