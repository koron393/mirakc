use log;
use serde_json;
use tokio::io::AsyncReadExt;

use crate::command_util;
use crate::epg::*;
use crate::error::Error;
use crate::models::*;
use crate::tuner;

pub struct ServiceScanner {
    command: String,
    channels: Vec<EpgChannel>,
}

// TODO: The following implementation has code clones similar to
//       ClockSynchronizer and EitCollector.

impl ServiceScanner {
    const LABEL: &'static str = "service-scanner";

    pub fn new(
        command: String,
        channels: Vec<EpgChannel>
    ) -> Self {
        ServiceScanner { command, channels }
    }

    pub async fn scan_services(self) -> Result<Vec<EpgService>, Error> {
        log::debug!("Scanning services...");

        let mut services = Vec::new();
        for channel in self.channels.iter() {
            services.append(&mut Self::scan_services_in_channel(
                &channel, &self.command).await?);
        }

        log::debug!("Found {} services", services.len());

        Ok(services)
    }

    async fn scan_services_in_channel(
        channel: &EpgChannel,
        command: &str,
    ) -> Result<Vec<EpgService>, Error> {
        log::debug!("Scanning services in {}...", channel.name);

        let user = TunerUser {
            info: TunerUserInfo::Job { name: Self::LABEL.to_string() },
            priority: -1,
        };

        let stream = tuner::start_streaming(
            channel.channel_type, channel.channel.clone(), user).await?;

        let (input, mut output) = command_util::spawn_pipeline(
            vec![command.to_string()], stream.id())?;

        let handle = tokio::spawn(stream.pipe(input));

        let mut buf = Vec::new();
        output.read_to_end(&mut buf).await?;

        // Explicitly dropping the output of the pipeline is needed.  The output
        // holds the child processes and it kills them when dropped.
        drop(output);

        // Wait for the task so that the tuner is released before a request for
        // streaming in the next iteration.
        let _ = handle.await;

        let services: Vec<TsService> = serde_json::from_slice(&buf)?;
        log::debug!("Found {} services in {}", services.len(), channel.name);

        Ok(services
           .into_iter()
           .map(|sv| EpgService::from((channel, &sv)))
           .collect())
    }
}
