#[cfg(feature = "test-support")]
use agent_client_protocol as acp;
use tokio::sync::{mpsc, oneshot};

use crate::Error;

/// An agent capable of starting sessions.
pub struct Agent {
    kind: Kind,
}

enum Kind {
    #[cfg(feature = "test-support")]
    Fake { reply: Vec<String> },
}

enum Command {
    Prompt {
        text: String,
        response: oneshot::Sender<Result<String, Error>>,
    },
    Close,
}

/// An active conversation with an agent.
pub struct Session {
    commands: mpsc::Sender<Command>,
    task: tokio::task::JoinHandle<Result<(), Error>>,
}

impl Agent {
    /// Start a session in the given working directory.
    ///
    /// # Errors
    /// Returns an error if the connection or session setup fails.
    pub async fn start_session(
        &self,
        working_directory: &std::path::Path,
    ) -> Result<Session, Error> {
        let _ = working_directory;
        match self.kind {
            #[cfg(feature = "test-support")]
            Kind::Fake { ref reply } => {
                let (client, server) = acp::Channel::duplex();
                let reply = reply.clone();
                let (commands, receiver) = mpsc::channel(8);
                let (ready, initialized) = oneshot::channel();
                let working_directory = working_directory.to_owned();
                let task = tokio::spawn(async move {
                    let server_task = tokio::spawn(crate::test_support::serve(server, reply));
                    let result = run_client(client, working_directory, receiver, ready).await;
                    server_task.abort();
                    let server_result = server_task.await;
                    if let Err(error) = server_result
                        && !error.is_cancelled()
                    {
                        return Err(Error::Communication {
                            message: error.to_string(),
                        });
                    }
                    result.map_err(|error| Error::Communication {
                        message: error.to_string(),
                    })
                });
                initialized.await.map_err(|error| Error::Communication {
                    message: error.to_string(),
                })??;
                Ok(Session { commands, task })
            }
        }
    }

    #[cfg(feature = "test-support")]
    pub(crate) fn fake(reply: Vec<String>) -> Self {
        Self {
            kind: Kind::Fake { reply },
        }
    }
}

impl Session {
    /// Send a prompt and collect the agent's reply.
    ///
    /// # Errors
    /// Returns an error if communication with the agent fails.
    pub async fn prompt(&mut self, text: &str) -> Result<String, Error> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Prompt {
                text: text.to_owned(),
                response: sender,
            })
            .await
            .map_err(|error| Error::Communication {
                message: error.to_string(),
            })?;

        receiver.await.map_err(|error| Error::Communication {
            message: error.to_string(),
        })?
    }

    /// Close the session and wait for the client to stop.
    ///
    /// # Errors
    /// Returns an error if the client cannot be stopped or reports a failure.
    pub async fn close(self) -> Result<(), Error> {
        let sent = self
            .commands
            .send(Command::Close)
            .await
            .map_err(|error| Error::Communication {
                message: error.to_string(),
            });
        let result = self.task.await.map_err(|error| Error::Communication {
            message: error.to_string(),
        })?;
        sent?;
        result
    }
}

#[cfg(feature = "test-support")]
async fn run_client(
    channel: acp::Channel,
    working_directory: std::path::PathBuf,
    mut commands: mpsc::Receiver<Command>,
    ready: oneshot::Sender<Result<(), Error>>,
) -> acp::Result<()> {
    acp::Client
        .v1()
        .connect_with(channel, async move |connection| {
            connection
                .send_request(acp::v1::schema::InitializeRequest::new(
                    acp::schema::ProtocolVersion::V1,
                ))
                .block_task()
                .await?;

            connection
                .build_session(&working_directory)
                .block_task()
                .run_until(async move |acp::v1::SessionSetup { mut session, .. }| {
                    let _ = ready.send(Ok(()));

                    while let Some(command) = commands.recv().await {
                        match command {
                            Command::Prompt { text, response } => {
                                let result = match session.send_prompt(text) {
                                    Ok(()) => session.read_to_string().await,
                                    Err(error) => Err(error),
                                }
                                .map_err(|error| {
                                    Error::Communication {
                                        message: error.to_string(),
                                    }
                                });
                                let _ = response.send(result);
                            }
                            Command::Close => break,
                        }
                    }
                    Ok(())
                })
                .await
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::{Command, Error, Session};
    use tokio::sync::mpsc;

    #[cfg(feature = "test-support")]
    #[tokio::test]
    async fn dropping_session_stops_client_and_server() -> Result<(), Box<dyn std::error::Error>> {
        let agent = super::Agent::fake(vec!["reply".to_owned()]);
        let session = agent.start_session(&std::env::current_dir()?).await?;
        let task = session.task.abort_handle();
        drop(session);
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while !task.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        Ok(())
    }

    #[cfg(feature = "test-support")]
    #[tokio::test]
    async fn close_waits_for_client_and_server() -> Result<(), Box<dyn std::error::Error>> {
        let agent = super::Agent::fake(vec!["reply".to_owned()]);
        let session = agent.start_session(&std::env::current_dir()?).await?;
        let task = session.task.abort_handle();
        session.close().await?;
        assert!(task.is_finished());
        Ok(())
    }

    #[tokio::test]
    async fn prompt_reports_disconnected_client() {
        let (commands, receiver) = mpsc::channel(1);
        drop(receiver);
        let task = tokio::spawn(async { Ok(()) });
        let mut session = Session { commands, task };
        assert!(matches!(
            session.prompt("hello").await,
            Err(Error::Communication { .. })
        ));
    }

    #[tokio::test]
    async fn prompt_reports_dropped_response() {
        let (commands, mut receiver) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            if let Some(Command::Prompt { response, .. }) = receiver.recv().await {
                drop(response);
            }
            Ok(())
        });
        let mut session = Session { commands, task };
        assert!(matches!(
            session.prompt("hello").await,
            Err(Error::Communication { .. })
        ));
    }

    #[tokio::test]
    async fn close_reports_disconnected_client() {
        let (commands, receiver) = mpsc::channel(1);
        drop(receiver);
        let task = tokio::spawn(async { Ok(()) });
        let session = Session { commands, task };
        assert!(matches!(
            session.close().await,
            Err(Error::Communication { .. })
        ));
    }
}
