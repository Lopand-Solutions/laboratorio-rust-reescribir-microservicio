//! El contrato: cada caso de herramientas/casos.json, con la respuesta que grabó el servicio de Node.
//! Corre sin red ni servidor: oneshot llama al router directo. Sirve para un CI sin servicios.
use axum::{
    body::{Body, to_bytes},
    http::{Request, header},
};
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Deserialize)]
struct Caso {
    metodo: String,
    ruta: String,
    esperado: Esperado,
}

#[derive(Deserialize)]
struct Esperado {
    codigo: u16,
    tipo: String,
    cuerpo: String,
}

#[tokio::test]
async fn responde_igual_que_node_en_todos_los_casos() {
    let texto = include_str!("../herramientas/casos.json");
    let casos: Vec<Caso> = serde_json::from_str(texto).unwrap();
    let mut diferencias = Vec::new();
    for caso in &casos {
        let req = Request::builder()
            .method(caso.metodo.as_str())
            .uri(caso.ruta.as_str())
            .body(Body::empty())
            .unwrap();
        let r = convertidor::app().oneshot(req).await.unwrap();
        let codigo = r.status().as_u16();
        let tipo = r
            .headers()
            .get(header::CONTENT_TYPE)
            .map_or("(sin content-type)", |v| v.to_str().unwrap())
            .to_owned();
        let cuerpo = to_bytes(r.into_body(), 64 * 1024).await.unwrap();
        let cuerpo = String::from_utf8(cuerpo.to_vec()).unwrap();
        let e = &caso.esperado;
        if (codigo, tipo.as_str(), cuerpo.as_str())
            != (e.codigo, e.tipo.as_str(), e.cuerpo.as_str())
        {
            diferencias.push(format!(
                "{} {}\n  node: {} {} {}\n  rust: {codigo} {tipo} {cuerpo}",
                caso.metodo, caso.ruta, e.codigo, e.tipo, e.cuerpo
            ));
        }
    }
    assert!(diferencias.is_empty(), "{}", diferencias.join("\n"));
}
