use std::ops::Add;
use std::time::Duration;
use tokio::time::Instant;
use crate::config::HealthcheckCmdConfig;
use crate::runner::CmdRunner;

pub struct HealthcheckCommand {
    timeout_millis: u64,
    command: CmdRunner,
}

impl HealthcheckCommand {
    pub fn new(conf: HealthcheckCmdConfig) -> anyhow::Result<Self> {
        Ok(Self {
            timeout_millis: conf.timeout_millis,
            command: CmdRunner::build(&conf.command, &conf.args, "/tmp")?,
        })
    }

    fn healthcheck_intervals(&self) -> Vec<Instant> {
        let mut intervals = vec![self.timeout_millis];
        let mut current_inteval = self.timeout_millis;
        while current_inteval > 1000 {
            intervals.push(current_inteval);
            current_inteval /= 2;
        }

        intervals.into_iter()
            .rev()
            .map(|interval_millis| Instant::now().add(Duration::from_millis(interval_millis)))
            .collect()
    }
    pub async fn wait_until_healthy(&mut self) -> anyhow::Result<()> {
        let mut intervals = self.healthcheck_intervals();
        while !self.command.run()?.status.success() {
            intervals.retain(|instant| instant > &Instant::now());
            if let Some(instant) = intervals.pop() {
                tokio::time::sleep_until(instant).await;
            }
        }
        Ok(())
    }
}