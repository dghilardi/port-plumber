use std::path::Path;
use std::process::{Child, Command, Stdio};

use anyhow::Result;

pub struct CmdRunner {
    command: Command,
    process: Option<Child>
}

impl CmdRunner {
    pub fn build(cmd_name: &str, args: &[String], dir: impl AsRef<Path>) -> Result<Self> {
        let mut command = Command::new(cmd_name);
        command.current_dir(dir);
        command.args(args);
        command.stdin(Stdio::piped());

        Ok(Self {
            command,
            process: None
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut process = self.command.spawn()?;
        let exit_status = process.wait()?;
        if exit_status.success() {
            Ok(())
        } else {
            anyhow::bail!("process exited with {exit_status}")
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let mut process = self.command.spawn()?;
        self.process = Some(process);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        let Some(ref mut process) = self.process else {
            return Ok(())
        };
        if let Some(_status) = process.try_wait()? {
            self.process = None;
            Ok(())
        } else {
            process.kill()?;
            let _status = process.wait()?;
            self.process = None;
            Ok(())
        }
    }

    pub fn is_running(&mut self) -> Result<bool> {
        let running = self.process
            .as_mut()
            .map(|p| p.try_wait().map(|out| out.is_none()))
            .transpose()?
            .unwrap_or(false);
        Ok(running)
    }
}