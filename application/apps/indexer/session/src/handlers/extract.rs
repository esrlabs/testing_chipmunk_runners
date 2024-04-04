use crate::{
    events::{NativeError, NativeErrorKind},
    operations::{OperationAPI, OperationInterface, OperationResult},
    progress::Severity,
    state::SessionStateAPI,
    unbound::signal::Signal,
};
use async_trait::async_trait;

use processor::search::{
    extractor::{ExtractedMatchValue, MatchesExtractor},
    filter::SearchFilter,
};
use std::path::{Path, PathBuf};

struct ExtractOperation {
    target_file_path: PathBuf,
    filters: Vec<SearchFilter>,
    signal: Signal,
}

#[async_trait]
impl OperationInterface for ExtractOperation {
    type Output = Vec<ExtractedMatchValue>;

    async fn execute(
        &self,
        _: &OperationAPI,
        state_api: &SessionStateAPI,
    ) -> OperationResult<Self::Output> {
        handle(&self.target_file_path, self.filters)
    }

    fn get_signal(&self) -> Signal {
        self.signal.clone()
    }
}

pub fn handle(
    target_file_path: &Path,
    filters: Vec<SearchFilter>,
) -> OperationResult<Vec<ExtractedMatchValue>> {
    let extractor = MatchesExtractor::new(target_file_path, filters);
    extractor
        .extract_matches()
        .map(Some)
        .map_err(|e| NativeError {
            severity: Severity::ERROR,
            kind: NativeErrorKind::OperationSearch,
            message: Some(format!(
                "Fail to execute extract search result operation. Error: {e}"
            )),
        })
}
