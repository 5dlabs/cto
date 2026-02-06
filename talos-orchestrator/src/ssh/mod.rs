use std::io::{Read, Write};
use std::path::Path;
use ssh2::Session;
use crate::error::Error;

pub struct SshClient {
    session: Session,
}

impl SshClient {
    /// Connect to SSH server
    pub fn connect(host: &str, username: &str, password: &str) -> Result<Self, Error> {
        let tcp = std::net::TcpStream::connect(format!("{}:22", host))
            .map_err(|e| Error::Ssh(e.to_string()))?;

        let mut session = Session::new().unwrap();
        session.set_tcp_stream(tcp);
        session.handshake().map_err(|e| Error::Ssh(e.to_string()))?;

        session
            .userauth_password(username, password)
            .map_err(|e| Error::Ssh(e.to_string()))?;

        Ok(Self { session })
    }

    /// Execute a command and return stdout
    pub fn execute(&self, command: &str) -> Result<String, Error> {
        let mut channel = self
            .session
            .channel_session()
            .map_err(|e| Error::Ssh(e.to_string()))?;

        channel.exec(command).map_err(|e| Error::Ssh(e.to_string()))?;

        let mut output = String::new();
        let mut buffer = [0u8; 4096];
        loop {
            let n = channel.read(&mut buffer).map_err(|e| Error::Ssh(e.to_string()))?;
            if n == 0 {
                break;
            }
            output.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
        
        channel.wait_close().map_err(|e| Error::Ssh(e.to_string()))?;

        Ok(output)
    }

    /// Upload file using SCP
    pub fn upload(&self, local_path: &Path, remote_path: &Path) -> Result<(), Error> {
        let mut remote_file = self
            .session
            .scp_send(remote_path, 0o644, local_path.metadata()?.len(), None)
            .map_err(|e| Error::Ssh(e.to_string()))?;

        let mut local_file = std::fs::File::open(local_path)?;

        let mut buffer = Vec::new();
        Read::read_to_end(&mut local_file, &mut buffer)?;

        remote_file.write_all(&buffer).map_err(|e| Error::Ssh(e.to_string()))?;
        remote_file.send_eof().map_err(|e| Error::Ssh(e.to_string()))?;
        remote_file.wait_eof().map_err(|e| Error::Ssh(e.to_string()))?;
        remote_file.close().map_err(|e| Error::Ssh(e.to_string()))?;

        Ok(())
    }

    /// rsync-style upload (fallback to SCP)
    pub fn rsync_upload(&self, local_path: &Path, remote_path: &str) -> Result<(), Error> {
        self.upload(local_path, Path::new(remote_path))
    }
}
