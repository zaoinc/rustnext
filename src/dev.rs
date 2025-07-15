#[cfg(feature = "dev")]
use crate::{App, Server};
#[cfg(feature = "dev")]
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event};
#[cfg(feature = "dev")]
use std::net::SocketAddr;
#[cfg(feature = "dev")]
use std::path::Path;
#[cfg(feature = "dev")]
use std::sync::mpsc::channel;
#[cfg(feature = "dev")]
use std::time::Duration;

#[cfg(feature = "dev")]
pub struct DevServer {
    app: App,
    addr: SocketAddr,
    watch_dir: String,
}

#[cfg(feature = "dev")]
impl DevServer {
    pub fn new(app: App, addr: SocketAddr, watch_dir: &str) -> Self {
        DevServer {
            app,
            addr,
            watch_dir: watch_dir.to_string(),
        }
    }

    pub async fn run_with_reload(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = channel();
        
        // Create watcher with proper event handler
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if let Err(e) = tx.send(event) {
                            eprintln!("Watch error: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            },
            notify::Config::default()
        )?;
        
        // Convert String to Path and watch
        watcher.watch(Path::new(&self.watch_dir), RecursiveMode::Recursive)?;

        println!("Development server starting with file watching on {}", self.watch_dir);
        
        let server = Server::new(self.app, self.addr);
        
        tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                println!("File changed: {:?}", event);
            }
        });

        server.run().await
    }
}
