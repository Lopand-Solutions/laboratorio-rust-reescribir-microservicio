//! El convertidor en Rust. Debe responder lo mismo que node/servicio.js, byte por byte:
//! mismo código, mismo content-type y mismo cuerpo.
use axum::{
    Router,
    extract::Query,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use serde_json::json;

pub const MONTO_MAXIMO: u64 = 100_000_000;

/// Tasas de ejemplo, no reales, en millonésimas: 18_500_000 es 18.5.
pub fn tasa(de: &str, a: &str) -> Option<u64> {
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

/// Redondeo a la mitad hacia arriba, en enteros, igual que Math.floor((m * t + 500000) / 1000000).
pub fn convertir_centavos(centavos: u64, tasa: u64) -> u64 {
    (centavos * tasa + 500_000) / 1_000_000
}

/// Lo mismo que /^\d{1,9}$/ y el rango de Node. Ojo: "+500".parse::<u64>() da Ok(500); por eso
/// revisamos los dígitos antes de convertir.
pub fn leer_monto(texto: &str) -> Option<u64> {
    if texto.is_empty() || texto.len() > 9 || !texto.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let monto: u64 = texto.parse().ok()?;
    (1..=MONTO_MAXIMO).contains(&monto).then_some(monto)
}

#[derive(Serialize)]
struct Conversion<'a> {
    de: &'a str,
    a: &'a str,
    monto_centavos: u64,
    tasa_millonesimas: u64,
    resultado_centavos: u64,
}

/// Node manda "application/json; charset=utf-8"; el Json de axum manda "application/json". Lo armamos a mano.
fn json(codigo: StatusCode, cuerpo: impl Serialize) -> Response {
    let texto = serde_json::to_string(&cuerpo).expect("JSON válido");
    let tipo = [(header::CONTENT_TYPE, "application/json; charset=utf-8")];
    (codigo, tipo, texto).into_response()
}

fn error(codigo: StatusCode, mensaje: &str) -> Response {
    json(codigo, json!({ "error": mensaje }))
}

/// Como URLSearchParams.get(): el primer valor si la clave se repite, y None si no está.
fn primero<'a>(pares: &'a [(String, String)], clave: &str) -> Option<&'a str> {
    pares
        .iter()
        .find(|(k, _)| k == clave)
        .map(|(_, v)| v.as_str())
}

async fn convertir(Query(pares): Query<Vec<(String, String)>>) -> Response {
    let de = primero(&pares, "de").unwrap_or("");
    let a = primero(&pares, "a").unwrap_or("");
    let monto = primero(&pares, "monto");
    // En Node, `!de` es verdadero también con de= (cadena vacía).
    let (false, false, Some(monto)) = (de.is_empty(), a.is_empty(), monto) else {
        return error(StatusCode::BAD_REQUEST, "faltan de, a o monto");
    };
    let Some(centavos) = leer_monto(monto) else {
        let mensaje = format!("monto debe ser un entero de centavos entre 1 y {MONTO_MAXIMO}");
        return error(StatusCode::BAD_REQUEST, &mensaje);
    };
    let Some(tasa) = tasa(de, a) else {
        return error(StatusCode::NOT_FOUND, &format!("no hay tasa de {de} a {a}"));
    };
    let conversion = Conversion {
        de,
        a,
        monto_centavos: centavos,
        tasa_millonesimas: tasa,
        resultado_centavos: convertir_centavos(centavos, tasa),
    };
    json(StatusCode::OK, conversion)
}

async fn salud() -> Response {
    json(StatusCode::OK, json!({ "estado": "ok" }))
}

pub fn app() -> Router {
    Router::new()
        .route("/salud", get(salud))
        .route("/convertir", get(convertir))
        // Sin esto, axum responde 404 y 405 con el cuerpo vacío.
        .fallback(|| async { error(StatusCode::NOT_FOUND, "no encontrado") })
        .method_not_allowed_fallback(|| async {
            error(StatusCode::METHOD_NOT_ALLOWED, "método no permitido")
        })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_monto_solo_acepta_digitos() {
        assert_eq!(leer_monto("10000"), Some(10_000));
        assert_eq!(leer_monto("007"), Some(7));
        for malo in [
            "",
            "0",
            "+500",
            "-5",
            "12.5",
            " 500",
            "100000001",
            "1000000000",
        ] {
            assert_eq!(leer_monto(malo), None, "debió rechazar {malo:?}");
        }
    }

    #[test]
    fn redondea_a_la_mitad_hacia_arriba() {
        // 7 centavos de USD a 18.5: 129.5 centavos de MXN, se redondea a 130.
        assert_eq!(convertir_centavos(7, 18_500_000), 130);
        assert_eq!(convertir_centavos(1, 1_086_957), 1);
        // El caso más grande cabe de sobra en u64.
        assert_eq!(convertir_centavos(MONTO_MAXIMO, 20_100_000), 2_010_000_000);
    }
}
