use std::time::Duration;

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::info;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{PlugCommand, PlugId, PlugTask, PowerState, SharedState};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, utoipa::IntoParams)]
pub struct QueryStatusParams {
    id: PlugId,
}

#[derive(Debug, Clone, Serialize, utoipa::ToResponse, utoipa::ToSchema)]
pub struct QueryStatusResponse {
    state: Option<PowerState>,
    lastseen: Option<chrono::DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "/api/query",
    params(QueryStatusParams),
    responses(
        (status = 200, body = QueryStatusResponse)
    )
)]
pub async fn query_status(
    State(s): State<SharedState>,
    Query(params): Query<QueryStatusParams>,
) -> Json<QueryStatusResponse> {
    if let Some(plug) = s.plugs.get(&params.id) {
        if plug.power_state == PowerState::Unknown {
            let (task, rx) = PlugTask::new(PlugCommand::QueryState);
            let _ = timeout(Duration::from_secs(10), async {
                let sent = plug.task_tx.send(task).await.is_ok();
                // avoids deadlocking
                drop(plug);
                if !sent {
                    tracing::warn!("Sending task to plug failed");
                    false
                } else {
                    match rx.await {
                        Ok(b) => b,
                        Err(_) => {
                            tracing::warn!("task dropped by plug");
                            false
                        }
                    }
                }
            })
            .await
            .is_ok_and(|i| i);
        }
        match s.plugs.get(&params.id) {
            Some(status) => Json(QueryStatusResponse {
                state: Some(status.power_state),
                lastseen: Some(status.last_seen),
            }),
            None => Json(QueryStatusResponse {
                state: None,
                lastseen: None,
            }),
        }
    } else {
        Json(QueryStatusResponse {
            state: None,
            lastseen: None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum PowerStateOption {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, utoipa::IntoParams)]
pub struct StateQuery {
    // Plug ID
    id: PlugId,
    // Desired plug power state
    state: PowerStateOption,
}

#[derive(Debug, Clone, Serialize, utoipa::ToResponse, utoipa::ToSchema)]
pub struct SetStateResponse {
    present: bool,
    success: bool,
}

#[utoipa::path(
    post,
    path = "/api/setstate",
    params(
        StateQuery
    ),
    responses(
        (status = 200, description = "Success", body = SetStateResponse),
    )
)]
pub async fn set_state(
    State(s): State<SharedState>,
    Query(query): Query<StateQuery>,
) -> Json<SetStateResponse> {
    info!("Turning {} {:?}", *query.id, &query.state);
    if let Some(plug) = s.plugs.get(&query.id) {
        let (task, rx) = PlugTask::new(match query.state {
            PowerStateOption::On => PlugCommand::TurnOn,
            PowerStateOption::Off => PlugCommand::TurnOff,
        });
        let success = timeout(Duration::from_secs(10), async {
            let sent = plug.task_tx.send(task).await.is_ok();
            // avoids deadlocking
            drop(plug);
            if !sent {
                tracing::warn!("Sending task to plug failed");
                false
            } else {
                match rx.await {
                    Ok(b) => b,
                    Err(_) => {
                        tracing::warn!("task dropped by plug");
                        false
                    }
                }
            }
        })
        .await
        .is_ok_and(|i| i);

        Json(SetStateResponse {
            present: true,
            success,
        })
    } else {
        Json(SetStateResponse {
            present: false,
            success: false,
        })
    }
}

#[derive(Serialize, ToSchema)]
pub struct ListResponse {
    plugs: Vec<PlugListInfo>,
}

#[derive(Serialize, ToSchema)]
pub struct PlugListInfo {
    id: PlugId,
    state: PowerState,
    last_seen: chrono::DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/api/list",
    responses(
        (status = 200, body = ListResponse)
    )
)]
pub async fn list_plugs(State(s): State<SharedState>) -> Json<ListResponse> {
    Json(ListResponse {
        plugs: s
            .plugs
            .iter()
            .map(|k| PlugListInfo {
                id: *k.key(),
                state: k.value().power_state,
                last_seen: k.value().last_seen,
            })
            .collect(),
    })
}

pub fn router() -> OpenApiRouter<SharedState> {
    OpenApiRouter::new()
        .routes(routes!(set_state))
        .routes(routes!(query_status))
        .routes(routes!(list_plugs))
}
