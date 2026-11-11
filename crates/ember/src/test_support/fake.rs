use crate::Agent;
use agent_client_protocol::{Agent as AcpAgent, Channel, Client, v1 as acp};

/// An in-memory agent that replies with fixed text.
pub struct FakeAgent {
    reply: Vec<String>,
    notification: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

#[derive(Clone, Copy)]
pub(crate) enum SessionSetup {
    Accept,
    #[cfg(test)]
    Reject,
}

pub(crate) async fn serve(
    channel: Channel,
    reply: Vec<String>,
    setup: SessionSetup,
    notification: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> agent_client_protocol::Result<()> {
    AcpAgent
        .v1()
        .on_receive_request(
            async move |request: acp::schema::InitializeRequest, responder, _connection| {
                responder.respond(acp::schema::InitializeResponse::new(
                    request.protocol_version,
                ))
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |_request: acp::schema::NewSessionRequest, responder, _connection| {
                match setup {
                    SessionSetup::Accept => {
                        responder.respond(acp::schema::NewSessionResponse::new("fake-session"))
                    }
                    #[cfg(test)]
                    SessionSetup::Reject => {
                        responder.respond_with_internal_error("scripted session rejection")
                    }
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            move |request: acp::schema::PromptRequest,
                  responder: acp::Responder<acp::schema::PromptResponse>,
                  connection: acp::ConnectionTo<Client>| {
                let reply = reply.clone();
                let notification = notification.clone();

                async move {
                    if let Some(sent) = notification {
                        // Real agents send extension notifications that the client may not know.
                        connection.send_notification(
                            agent_client_protocol::UntypedMessage::new("_auth/status_update", ())?,
                        )?;
                        sent.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    for chunk in reply {
                        let content =
                            acp::schema::ContentBlock::Text(acp::schema::TextContent::new(chunk));
                        let update = acp::schema::SessionUpdate::AgentMessageChunk(
                            acp::schema::ContentChunk::new(content),
                        );
                        connection.send_notification(acp::schema::SessionNotification::new(
                            request.session_id.clone(),
                            update,
                        ))?;
                    }

                    responder.respond(acp::schema::PromptResponse::new(
                        acp::schema::StopReason::EndTurn,
                    ))
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_to(channel)
        .await
}

impl FakeAgent {
    /// Create a fake agent with the given reply.
    #[must_use]
    pub fn new(reply: impl Into<String>) -> Self {
        Self {
            reply: vec![reply.into()],
            notification: None,
        }
    }

    /// Create a fake that streams the given text chunks for each reply.
    #[must_use]
    pub fn with_chunks(chunks: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            reply: chunks.into_iter().map(Into::into).collect(),
            notification: None,
        }
    }

    /// Send an unknown extension notification and mark the flag once sent.
    #[must_use]
    pub fn with_unknown_notification(
        mut self,
        sent: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        self.notification = Some(sent);
        self
    }

    /// Convert the fake into an agent that can start sessions.
    #[must_use]
    pub fn into_agent(self) -> Agent {
        Agent::fake(self.reply, SessionSetup::Accept, self.notification)
    }
}
