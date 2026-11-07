//! El convertidor en Rust: primera versión, traducida a ojo desde node/servicio.js.
use axum::{
    Json, Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

const MONTO_MAXIMO: u64 = 100_000_000;

fn tasa(de: &str, a: &str) -> Option<u64> {
    match (de, a) {
        ("USD", "MXN") => Some(18_500_000),
        ("MXN", "USD") => Some(54_054),
        ("EUR", "MXN") => Some(20_100_000),
        ("MXN", "EUR") => Some(49_751),
        ("USD", "EUR") => Some(920_000),
        ("EUR", "USD") => Some(1_086_957),
        _ => None,
    }
}

#[derive(Deserialize)]
pub struct Parametros {
    de: Option<String>,
    a: Option<String>,
    monto: Option<String>,
}

#[derive(Serialize)]
struct Conversion {
    de: String,
    a: String,
    monto_centavos: u64,
    tasa_millonesimas: u64,
    resultado_centavos: u64,
}

fn error(codigo: StatusCode, mensaje: String) -> Response {
    (codigo, Json(json!({ "error": mensaje }))).into_response()
}

async fn convertir(Query(p): Query<Parametros>) -> Response {
    let (Some(de), Some(a), Some(monto)) = (p.de, p.a, p.monto) else {
        return error(StatusCode::BAD_REQUEST, "faltan de, a o monto".into());
    };
    let centavos = match monto.parse::<u64>() {
        Ok(m) if (1..=MONTO_MAXIMO).contains(&m) => m,
        _ => {
            return error(
                StatusCode::BAD_REQUEST,
                format!("monto debe ser un entero de centavos entre 1 y {MONTO_MAXIMO}"),
            );
        }
    };
    let Some(tasa) = tasa(&de, &a) else {
        return error(StatusCode::NOT_FOUND, format!("no hay tasa de {de} a {a}"));
    };
    Json(Conversion {
        de,
        a,
        monto_centavos: centavos,
        tasa_millonesimas: tasa,
        resultado_centavos: (centavos * tasa + 500_000) / 1_000_000,
    })
    .into_response()
}

async fn salud() -> Response {
    Json(json!({ "estado": "ok" })).into_response()
}

pub fn app() -> Router {
    Router::new()
        .route("/salud", get(salud))
        .route("/convertir", get(convertir))
}
