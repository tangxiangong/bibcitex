//! Private per-user socket with a held file lock; secondary launches only send commands.
use crate::Command;

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{
        fs::{self, File, OpenOptions},
        io::{Read, Write},
        os::unix::{
            fs::{OpenOptionsExt, PermissionsExt},
            net::{UnixListener, UnixStream},
        },
        path::PathBuf,
        time::Duration,
    };

    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    pub struct Instance {
        _lock: File,
        socket: PathBuf,
        stop: Arc<AtomicBool>,
        worker: Option<std::thread::JoinHandle<()>>,
    }
    impl Drop for Instance {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Release);
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
            let _ = fs::remove_file(&self.socket);
        }
    }

    pub fn acquire(
        command: &Command,
        sender: async_channel::Sender<Command>,
    ) -> Result<Option<Instance>, String> {
        let directory = dirs::runtime_dir()
            .or_else(dirs::cache_dir)
            .ok_or("No user runtime directory")?
            .join("bibcitex-session");
        acquire_in(directory, command, sender)
    }

    fn acquire_in(
        directory: PathBuf,
        command: &Command,
        sender: async_channel::Sender<Command>,
    ) -> Result<Option<Instance>, String> {
        fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        if fs::symlink_metadata(&directory)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("Runtime directory must not be a symlink".into());
        }
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(directory.join("instance.lock"))
            .map_err(|e| e.to_string())?;
        let socket = directory.join("instance.sock");
        match lock.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                let message = serde_json::to_vec(command).map_err(|e| e.to_string())?;
                for _ in 0..50 {
                    if let Ok(mut stream) = UnixStream::connect(&socket) {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .map_err(|e| e.to_string())?;
                        stream
                            .set_write_timeout(Some(Duration::from_secs(2)))
                            .map_err(|e| e.to_string())?;
                        stream.write_all(&message).map_err(|e| e.to_string())?;
                        stream
                            .shutdown(std::net::Shutdown::Write)
                            .map_err(|e| e.to_string())?;
                        let mut ack = [0];
                        stream.read_exact(&mut ack).map_err(|e| e.to_string())?;
                        return if ack[0] == 1 {
                            Ok(None)
                        } else {
                            Err("The running instance rejected activation".into())
                        };
                    }
                    std::thread::sleep(Duration::from_millis(40));
                }
                return Err("The running instance did not accept activation".into());
            }
            Err(error) => return Err(error.to_string()),
        }
        if socket.exists() {
            fs::remove_file(&socket).map_err(|e| e.to_string())?;
        }
        let listener = UnixListener::bind(&socket).map_err(|e| e.to_string())?;
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let worker = std::thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                let mut stream = match listener.accept() {
                    Ok((stream, _)) => stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(_) => break,
                };
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                let mut bytes = Vec::new();
                let valid =
                    (&mut stream).take(1025).read_to_end(&mut bytes).is_ok() && bytes.len() <= 1024;
                let accepted = valid
                    && serde_json::from_slice::<Command>(&bytes).is_ok_and(|command| {
                        matches!(
                            command,
                            Command::Main
                                | Command::HelperToken(_)
                                | Command::MainToken(_)
                                | Command::Helper
                                | Command::Tray
                                | Command::Settings
                                | Command::Quit
                        ) && sender.try_send(command).is_ok()
                    });
                let _ = stream.write_all(&[u8::from(accepted)]);
            }
        });
        Ok(Some(Instance {
            _lock: lock,
            socket,
            stop,
            worker: Some(worker),
        }))
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn secondary_launch_forwards_and_clean_shutdown_releases_lock() {
            let dir = tempfile::tempdir().unwrap();
            let (send, receive) = async_channel::bounded(8);
            let primary = acquire_in(dir.path().join("instance"), &Command::Main, send.clone())
                .unwrap()
                .unwrap();
            assert!(
                acquire_in(
                    dir.path().join("instance"),
                    &Command::HelperToken("token".into()),
                    send.clone()
                )
                .unwrap()
                .is_none()
            );
            assert!(
                matches!(receive.recv_blocking().unwrap(), Command::HelperToken(token) if token == "token")
            );
            let socket = primary.socket.clone();
            drop(primary);
            assert!(!socket.exists());
            assert!(
                acquire_in(dir.path().join("instance"), &Command::Main, send)
                    .unwrap()
                    .is_some()
            );
        }
        #[test]
        fn private_socket_rejects_internal_commands_from_secondary_processes() {
            let dir = tempfile::tempdir().unwrap();
            let (send, receive) = async_channel::bounded(8);
            let primary = acquire_in(dir.path().join("instance"), &Command::Main, send)
                .unwrap()
                .unwrap();
            let mut stream = UnixStream::connect(&primary.socket).unwrap();
            stream.write_all(br#"{"Error":"injected"}"#).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut ack = [1];
            stream.read_exact(&mut ack).unwrap();
            assert_eq!(ack, [0]);
            assert!(receive.try_recv().is_err());
        }
    }
}
#[cfg(unix)]
pub use unix::{Instance, acquire};

#[cfg(not(unix))]
pub struct Instance;
#[cfg(not(unix))]
pub fn acquire(_: &Command, _: async_channel::Sender<Command>) -> Result<Option<Instance>, String> {
    Ok(Some(Instance))
}
