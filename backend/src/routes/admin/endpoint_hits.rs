use itertools::Itertools;

use crate::db;
use crate::routes::imports::*;
use crate::startup::UserConnectInfo;
use axum::extract::connect_info::ConnectInfo;

#[allow(unused)]
pub async fn endpoint_hits(
    session: ReadableSession,
    Extension(db): Extension<cozo::DbInstance>,
) -> ApiResult<Json<Vec<interfacing::EndpointHit>>> {
    if get_env().prod() {
        reject_anonymous_users(&session)?;
    }
    let result = db::q::find_endpoint_hits(&db)?;
    Ok(Json(result))
}

#[derive(serde::Serialize)]
struct IpToHit {
    hashed_ip: String,
    hits: Vec<interfacing::EndpointHit>,
}

#[allow(unused)]
pub async fn endpoint_hits_grouped(
    session: ReadableSession,
    Extension(db): Extension<cozo::DbInstance>,
) -> ApiResult<impl IntoResponse> {
    if get_env().prod() {
        reject_anonymous_users(&session)?;
    }
    let result = db::q::find_endpoint_hits(&db)?;

    let result = result
        .into_iter()
        .sorted_by_key(|v| v.timestamp())
        .rev()
        .group_by(|v| v.hashed_ip.clone())
        .into_iter()
        .map(|(hashed_ip, group)| IpToHit {
            hashed_ip,
            hits: group.into_iter().collect_vec(),
        })
        .collect_vec();

    Ok(Json(result))
}

#[allow(unused)]
pub async fn frontend_endpoint_hit(
    Extension(db): Extension<cozo::DbInstance>,
    ConnectInfo(con_info): ConnectInfo<UserConnectInfo>,
    Json(value): Json<interfacing::FrontendEndpointHit>,
) -> ApiResult<()> {
    let system_time = interfacing::EndpointHit::formatted_now();

    let hashed_ip = if get_env().local() {
        con_info.remote_addr.ip().to_string()
    } else {
        interfacing::EndpointHit::hash_ip(con_info.remote_addr.ip())
    };

    let hit = interfacing::EndpointHit {
        hashed_ip,
        endpoint: value.endpoint,
        method: "GET".into(),
        status: value.status,
        timestamp: system_time,
    };

    db::q::put_endpoint_hit(&db, hit)?;
    Ok(())
}
