/// An error encountered while communicating with an agent.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Communication with the agent failed.
    #[error("agent communication failed")]
    Communication {
        /// The underlying communication failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// The agent exited before session setup completed.
    #[error("agent exited during session setup")]
    SetupAborted,
}

impl Error {
    pub(crate) fn communication(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Communication {
            source: Box::new(error),
        }
    }
}
