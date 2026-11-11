use agent_client_protocol as acp;
use tokio::sync::{mpsc, oneshot};

use crate::Error;

type Connect = dyn Fn() -> (acp::Channel, tokio::task::JoinHandle<acp::Result<()>>) + Send + Sync;

/// An agent capable of starting sessions.
pub struct Agent {
    connect: Box<Connect>,
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
        let (client, server_task) = (self.connect)();
        let (commands, receiver) = mpsc::channel(8);
        let (ready, initialized) = oneshot::channel();
        let working_directory = working_directory.to_owned();
        let task = tokio::spawn(async move {
            let result = run_client(client, working_directory, receiver, ready).await;
            server_task.abort();
            let server_result = server_task.await;
            if let Err(error) = server_result
                && !error.is_cancelled()
            {
                return Err(Error::communication(error));
            }
            result.map_err(Error::communication)
        });
        if initialized.await.is_err() {
            return Err(match task.await {
                Ok(Err(error)) => error,
                Ok(Ok(())) => Error::SetupAborted,
                Err(error) => Error::communication(error),
            });
        }
        Ok(Session { commands, task })
    }

    #[cfg(feature = "test-support")]
    pub(crate) fn fake(
        reply: Vec<String>,
        setup: crate::test_support::SessionSetup,
        notification: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    ) -> Self {
        Self {
            connect: Box::new(move || {
                let (client, server) = acp::Channel::duplex();
                let task = tokio::spawn(crate::test_support::serve(
                    server,
                    reply.clone(),
                    setup,
                    notification.clone(),
                ));
                (client, task)
            }),
        }
    }
}

impl Session {
    /// Send a prompt and collect the agent's reply.
    ///
    /// The mutable borrow permits one prompt at a time. If this future is
    /// dropped after sending, the agent still finishes the turn; its reply is
    /// discarded and later prompts wait for it to finish.
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
            .map_err(Error::communication)?;

        receiver.await.map_err(Error::communication)?
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
            .map_err(Error::communication);
        let result = self.task.await.map_err(Error::communication)?;
        sent?;
        result
    }
}

async fn run_client(
    channel: acp::Channel,
    working_directory: std::path::PathBuf,
    commands: mpsc::Receiver<Command>,
    ready: oneshot::Sender<()>,
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

            let acp::v1::SessionSetup { mut session, .. } = connection
                .build_session(&working_directory)
                .block_task()
                .start_session()
                .await?;
            let _ = ready.send(());
            serve_commands(&mut session, commands).await;
            Ok(())
        })
        .await
}

async fn serve_commands(
    session: &mut acp::v1::ActiveSession<'_, acp::Agent>,
    mut commands: mpsc::Receiver<Command>,
) {
    while let Some(command) = commands.recv().await {
        match command {
            Command::Prompt { text, response } => {
                let _ = response.send(prompt_once(session, text).await);
            }
            Command::Close => break,
        }
    }
}

async fn prompt_once(
    session: &mut acp::v1::ActiveSession<'_, acp::Agent>,
    text: String,
) -> Result<String, Error> {
    session.send_prompt(text).map_err(Error::communication)?;
    session.read_to_string().await.map_err(Error::communication)
}

#[cfg(test)]
mod tests {
    use super::{Command, Error, Session};
    use tokio::sync::mpsc;

    #[cfg(feature = "test-support")]
    fn tracked_agent(token: std::sync::Arc<()>) -> super::Agent {
        super::Agent {
            connect: Box::new(move || {
                let (client, server) = super::acp::Channel::duplex();
                let token = token.clone();
                let task = tokio::spawn(async move {
                    let result = crate::test_support::serve(
                        server,
                        vec!["reply".to_owned()],
                        crate::test_support::SessionSetup::Accept,
                        None,
                    )
                    .await;
                    drop(token);
                    result
                });
                (client, task)
            }),
        }
    }

    #[cfg(feature = "test-support")]
    #[tokio::test]
    async fn setup_preserves_the_agents_rejection() -> Result<(), Box<dyn std::error::Error>> {
        let agent = super::Agent::fake(vec![], crate::test_support::SessionSetup::Reject, None);
        let result = agent.start_session(&std::env::current_dir()?).await;
        match result {
            Err(error) => {
                assert_eq!(error.to_string(), "agent communication failed");
                let source = std::error::Error::source(&error).ok_or("missing error source")?;
                assert!(source.to_string().contains("scripted session rejection"));
            }
            Ok(_) => return Err("session should have been rejected".into()),
        }
        Ok(())
    }

    #[cfg(feature = "test-support")]
    #[tokio::test]
    async fn dropping_session_stops_client_and_server() -> Result<(), Box<dyn std::error::Error>> {
        let token = std::sync::Arc::new(());
        let agent = tracked_agent(token.clone());
        let session = agent.start_session(&std::env::current_dir()?).await?;
        drop(agent);
        assert_eq!(std::sync::Arc::strong_count(&token), 2);
        let task = session.task.abort_handle();
        drop(session);
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while !task.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        assert_eq!(std::sync::Arc::strong_count(&token), 1);
        Ok(())
    }

    #[cfg(feature = "test-support")]
    #[tokio::test]
    async fn close_waits_for_client_and_server() -> Result<(), Box<dyn std::error::Error>> {
        let token = std::sync::Arc::new(());
        let agent = tracked_agent(token.clone());
        let session = agent.start_session(&std::env::current_dir()?).await?;
        drop(agent);
        assert_eq!(std::sync::Arc::strong_count(&token), 2);
        session.close().await?;
        assert_eq!(std::sync::Arc::strong_count(&token), 1);
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
