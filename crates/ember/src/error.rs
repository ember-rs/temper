/// An error encountered while communicating with an agent.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Communication with the agent failed.
    #[error("agent communication failed: {message}")]
    Communication {
        /// Details of the communication failure.
        message: String,
    },
}
