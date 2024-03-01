use super::CommandOutcome;
use crate::{events::ComputationError, unbound::signal::Signal};
use dlt_core::statistics::{collect_dlt_stats, StatisticInfo};
use std::path::Path;
use tokio::select;

pub async fn stats(
    files: Vec<String>,
    signal: Signal,
) -> Result<CommandOutcome<String>, ComputationError> {
    select! {
        _ = signal.cancelled() => {
            Ok(CommandOutcome::Cancelled)
        }
        res = async {
            let mut stat = StatisticInfo::new();
            let mut error: Option<String> = None;
            files.iter().for_each(|file| {
                if error.is_some() {
                    return;
                }
                match collect_dlt_stats(Path::new(&file), signal.token()) {
                    Ok(res) => {
                        stat.merge(res);
                    }
                    Err(err) => {
                        error = Some(err.to_string());
                    }
                }
            });
            if let Some(err) = error {
                Err(ComputationError::IoOperation(err))
            } else {
                match serde_json::to_string(&stat) {
                    Ok(res) => Ok(CommandOutcome::Finished(res)),
                    Err(err) => Err(ComputationError::IoOperation(err.to_string())),
                }
            }
        } => {
            res
        }
    }
}
