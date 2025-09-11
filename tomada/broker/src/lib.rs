pub mod api;
mod broker;

use std::{ops::Deref, sync::Arc};

pub use broker::*;
use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{Receiver as MpscReceiver, Sender as MpscSender},
    oneshot::{Receiver, Sender as OneshotSender},
};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct PlugId(Uuid);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum PowerState {
    On,
    Off,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlugCommand {
    TurnOn,
    TurnOff,
}

#[derive(Debug)]
pub struct PlugTask {
    completion: OneshotSender<bool>,
    command: PlugCommand,
}

pub type TaskTx = MpscSender<PlugTask>;
pub type TaskRx = MpscReceiver<PlugTask>;

#[derive(Debug, Clone)]
pub struct PlugState {
    last_seen: chrono::DateTime<Utc>,
    power_state: PowerState,
    task_tx: TaskTx,
}

#[derive(Debug, Clone, Default)]
pub struct SharedState {
    plugs: Arc<DashMap<PlugId, PlugState>>,
}

impl From<Uuid> for PlugId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl Deref for PlugId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PlugTask {
    pub fn new(command: PlugCommand) -> (Self, Receiver<bool>) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        (
            Self {
                command,
                completion: tx,
            },
            rx,
        )
    }

    pub fn command(&self) -> PlugCommand {
        self.command
    }

    pub fn complete(self, success: bool) {
        let _ = self.completion.send(success);
    }
}
