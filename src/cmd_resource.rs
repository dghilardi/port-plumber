use std::time::Duration;

use crate::config::ResourceConfig;
use crate::runner::CmdRunner;

pub enum CmdResource {
    Empty,
    Command {
        runner: CmdRunner,
        warmup: Duration,
    }
}

impl TryFrom<Option<&ResourceConfig>> for CmdResource {
    type Error = anyhow::Error;

    fn try_from(value: Option<&ResourceConfig>) -> Result<Self, Self::Error> {
        let Some(cfg) = value else {
            return Ok(Self::Empty)
        };
        Ok(Self::Command {
            runner: CmdRunner::build(&cfg.setup.command, &cfg.setup.args, &cfg.setup.workingdir)?,
            warmup: Duration::from_millis(cfg.warmup_millis),
        })
    }
}

impl CmdResource {
    pub async fn ensure_running(&mut self) -> anyhow::Result<()> {
        let Self::Command { runner, warmup } = self else {
            return Ok(())
        };
        if !runner.is_running()? {
            log::debug!("spawning command");
            runner.start()?;
            tokio::time::sleep(*warmup).await;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn ensure_stopped(&mut self) -> anyhow::Result<()> {
        let Self::Command { runner, .. } = self else {
            return Ok(())
        };
        if runner.is_running()? {
            log::debug!("stopping command");
            runner.stop()
        } else {
            Ok(())
        }
    }
}