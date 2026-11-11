/// An error encountered while communicating with an agent.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Communication with the agent failed.
    #[error("agent communication failed: {source}")]
    Communication {
        /// The underlying communication failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl Error {
    pub(crate) fn communication(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Communication {
            source: Box::new(error),
        }
    }
}
