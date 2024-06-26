use hyper::HeaderMap;
use itertools::Itertools;

use crate::db;
use crate::routes::imports::*;
use crate::startup::ip_address;

#[allow(unused)]
pub async fn endpoint_hits(
    session: ReadableSession,
    Extension(db): Extension<cozo::DbInstance>,
    Extension(conf): Extension<Conf>,
) -> ApiResult<Json<Vec<interfacing::EndpointHit>>> {
    if conf.env.prod() {
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
    Extension(conf): Extension<Conf>,
) -> ApiResult<impl IntoResponse> {
    if conf.env.prod() {
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

pub async fn frontend_endpoint_hit(
    Extension(db): Extension<cozo::DbInstance>,
    Extension(conf): Extension<Conf>,
    h: HeaderMap,
    Json(value): Json<interfacing::FrontendEndpointHit>,
) -> ApiResult<()> {
    let system_time = interfacing::EndpointHit::formatted_now();

    let ip = ip_address(&h);
    let hashed_ip = hash_ip(ip, &conf);

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

pub async fn github_hit(
    Extension(db): Extension<cozo::DbInstance>,
    Extension(conf): Extension<Conf>,
    h: HeaderMap,
) -> ApiResult<StatusCode> {
    let system_time = interfacing::EndpointHit::formatted_now();
    let ip = ip_address(&h);
    let hashed_ip = hash_ip(ip, &conf);

    let hit = interfacing::EndpointHit {
        hashed_ip,
        endpoint: "https://github.com/phantie".into(),
        method: "GET".into(),
        status: 200,
        timestamp: system_time,
    };

    db::q::put_endpoint_hit(&db, hit)?;
    Ok(StatusCode::NOT_FOUND)
}

pub async fn wsite_github_hit(
    Extension(db): Extension<cozo::DbInstance>,
    Extension(conf): Extension<Conf>,
    h: HeaderMap,
) -> ApiResult<StatusCode> {
    let system_time = interfacing::EndpointHit::formatted_now();
    let ip = ip_address(&h);
    let hashed_ip = hash_ip(ip, &conf);

    let hit = interfacing::EndpointHit {
        hashed_ip,
        endpoint: "https://github.com/phantie/wsite".into(),
        method: "GET".into(),
        status: 200,
        timestamp: system_time,
    };

    db::q::put_endpoint_hit(&db, hit)?;
    Ok(StatusCode::NOT_FOUND)
}

fn hash_ip(ip: std::net::IpAddr, conf: &Conf) -> String {
    if conf.env.local() {
        ip.to_string()
    } else {
        interfacing::EndpointHit::hash_ip(ip)
    }
}
