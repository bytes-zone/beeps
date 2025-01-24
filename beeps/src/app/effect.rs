use super::Action;
use crate::config::Config;
use beeps_core::{
    sync::{self, login, register, Client},
    Document, Replica,
};
use notify_rust::Notification;
use tokio::{fs, io};

/// Connections to external services that effect use. We keep these around to
/// have some level of connection sharing for the app as a whole.
pub struct EffectContext {
    /// an HTTP client with reqwest
    http: reqwest::Client,
}

impl EffectContext {
    /// Get a new `EffectConnections`
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

/// Things that can happen as a result of user input. Side effects!
#[derive(Debug)]
pub enum Effect {
    /// Save replica to disk
    SaveReplica(Replica),

    /// Save sync client auth to disk
    SaveSyncClientAuth(Client),

    /// Notify that a new ping is available
    NotifyAboutNewPing,

    /// Register a new account on the server and log into it
    Register(Client, register::Req),

    /// Log in to an existing account.
    LogIn(Client, login::Req),

    /// Check login status
    WhoAmI(Client),

    /// Push our replica to the server
    Push(Client, Document),
}

impl Effect {
    /// Perform the side-effectful portions of this effect, returning the next
    /// `Action` the application needs to handle
    pub async fn run(self, conn: &EffectContext, config: &Config) -> Option<Action> {
        match self.run_inner(conn, config).await {
            Ok(action) => action,
            Err(problem) => {
                tracing::error!(?problem, "problem running effect");
                Some(Action::Problem(problem.to_string()))
            }
        }
    }

    /// The actual implementation of `run`, but with a `Result` wrapper to make
    /// it more ergonomic to write.
    async fn run_inner(
        self,
        conn: &EffectContext,
        config: &Config,
    ) -> Result<Option<Action>, Problem> {
        match self {
            Self::SaveReplica(replica) => {
                tracing::debug!("saving replica");

                let base = config.data_dir();
                fs::create_dir_all(&base).await?;

                let store = base.join("store.json");

                let data = serde_json::to_vec(&replica)?;
                fs::write(&store, &data).await?;

                Ok(Some(Action::SavedReplica))
            }

            Self::SaveSyncClientAuth(client) => {
                tracing::info!("saving client auth");

                let base = config.data_dir();
                fs::create_dir_all(&base).await?;

                let store = base.join("auth.json");

                let data = serde_json::to_vec(&client)?;
                fs::write(&store, &data).await?;

                Ok(Some(Action::SavedSyncClientAuth))
            }

            Self::NotifyAboutNewPing => {
                tracing::debug!("notifying about new ping");

                // We don't care if the notification failed to show.
                let _ = Notification::new()
                    .summary("New ping!")
                    .body("What are you up to? Tag it!")
                    .show();

                Ok(None)
            }

            Self::Register(mut client, req) => {
                tracing::info!("registering");

                let resp = client.register(&conn.http, &req).await?;

                client.auth = Some(resp.jwt);

                Ok(Some(Action::LoggedIn(client)))
            }

            Self::LogIn(mut client, req) => {
                tracing::info!("logging in");

                let resp = client.login(&conn.http, &req).await?;

                client.auth = Some(resp.jwt);

                Ok(Some(Action::LoggedIn(client)))
            }

            Self::WhoAmI(client) => {
                tracing::info!("checking whoami");

                let resp = client.whoami(&conn.http).await?;

                Ok(Some(Action::GotWhoAmI(resp)))
            }

            Self::Push(client, document) => {
                tracing::info!("pushing document");

                let _ = client.push(&conn.http, &document).await?;

                Ok(Some(Action::Pushed))
            }
        }
    }
}

/// Problems that can happen while running an `Effect`.
#[derive(Debug, thiserror::Error)]
pub enum Problem {
    /// We had a problem writing to disk, for example with permissions or
    /// missing files.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    /// We had a problem loading or saving JSON.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// We had a problem communicating with the server, for example due to a bad
    /// URL or expired credentials.
    #[error("Problem communicating with the server: {0}")]
    Server(#[from] sync::Error),
}
