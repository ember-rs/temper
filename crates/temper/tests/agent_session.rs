//! Integration tests for agent sessions through Temper's dependency on Ember.
use ember::test_support::FakeAgent;

#[tokio::test]
async fn scripted_agent_replies_to_multiple_prompts() -> Result<(), Box<dyn std::error::Error>> {
    let agent = FakeAgent::new("Hello from the fake").into_agent();
    let mut session = agent.start_session(&std::env::current_dir()?).await?;

    assert_eq!(session.prompt("Hello").await?, "Hello from the fake");
    assert_eq!(session.prompt("Hello again").await?, "Hello from the fake");

    session.close().await?;
    Ok(())
}

#[tokio::test]
async fn session_can_close_without_a_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let agent = FakeAgent::new("unused").into_agent();
    agent
        .start_session(&std::env::current_dir()?)
        .await?
        .close()
        .await?;
    Ok(())
}

#[tokio::test]
async fn fake_agent_preserves_an_empty_reply() -> Result<(), Box<dyn std::error::Error>> {
    let agent = FakeAgent::new("").into_agent();
    let mut session = agent.start_session(&std::env::current_dir()?).await?;
    assert_eq!(session.prompt("").await?, "");
    session.close().await?;
    Ok(())
}

#[tokio::test]
async fn streamed_chunks_are_concatenated_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let agent = FakeAgent::with_chunks(["first", " ", "second"]).into_agent();
    let mut session = agent.start_session(&std::env::current_dir()?).await?;
    assert_eq!(session.prompt("hello").await?, "first second");
    session.close().await?;
    Ok(())
}

#[tokio::test]
async fn unknown_extension_notification_does_not_interrupt_the_turn()
-> Result<(), Box<dyn std::error::Error>> {
    let agent = FakeAgent::new("reply").into_agent();
    let mut session = agent.start_session(&std::env::current_dir()?).await?;
    assert_eq!(session.prompt("hello").await?, "reply");
    session.close().await?;
    Ok(())
}
