use crate::Agent;
use agent_client_protocol::{Agent as AcpAgent, Channel, Client, v1 as acp};

/// An in-memory agent that replies with fixed text.
pub struct FakeAgent {
    reply: Vec<String>,
}

pub(crate) async fn serve(
    channel: Channel,
    reply: Vec<String>,
    reject_session: bool,
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
                if reject_session {
                    return responder.respond_with_internal_error("scripted session rejection");
                }
                responder.respond(acp::schema::NewSessionResponse::new("fake-session"))
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            move |request: acp::schema::PromptRequest,
                  responder: acp::Responder<acp::schema::PromptResponse>,
                  connection: acp::ConnectionTo<Client>| {
                let reply = reply.clone();

                async move {
                    // Real agents send extension notifications that the client may not know.
                    connection.send_notification(agent_client_protocol::UntypedMessage::new(
                        "_auth/status_update",
                        (),
                    )?)?;
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
        }
    }

    /// Create a fake that streams the given text chunks for each reply.
    #[must_use]
    pub fn with_chunks(chunks: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            reply: chunks.into_iter().map(Into::into).collect(),
        }
    }

    /// Convert the fake into an agent that can start sessions.
    #[must_use]
    pub fn into_agent(self) -> Agent {
        Agent::fake(self.reply, false)
    }
}
