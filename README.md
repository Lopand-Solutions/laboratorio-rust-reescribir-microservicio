# Laboratorio: reescribe un microservicio de Node en Rust y mídelo

Repositorio del laboratorio de [Lopand Academy](https://academy.lopandsolutions.com/tutoriales/laboratorio-rust-reescribir-microservicio).
Un convertidor de monedas pequeño escrito en Node (`node/servicio.js`, sin dependencias) y su reescritura en Rust con
Axum, que debe responder **lo mismo, byte por byte**: código, `content-type` y cuerpo. Con un comparador, pruebas
que corren sin red y un script que mide a los dos con la misma carga.

> Las tasas de cambio del servicio son de ejemplo, no reales.

## Ramas

- **`inicio`**: el servicio de Node, los casos con sus respuestas grabadas, el comparador, el script de medición, las
  pruebas y una primera versión en Rust «traducida a ojo» que todavía responde distinto.
- **`main`**: la versión en Rust que pasa los 21 casos, tal como está en la lección.

```bash
git clone -b inicio https://github.com/Lopand-Solutions/laboratorio-rust-reescribir-microservicio.git convertidor
cd convertidor
```

## Córrelo

En una terminal, el original; en otra, el de Rust; en una tercera, el comparador:

```bash
PUERTO=3001 node node/servicio.js
PUERTO=3002 cargo run
node herramientas/comparar.mjs http://127.0.0.1:3001 http://127.0.0.1:3002
```

Las pruebas no necesitan red ni servidores:

```bash
cargo test
```

## Mídelo

```bash
cargo install oha
./herramientas/medir.sh
```

Compila en release, levanta cada servicio, le da 2 s de calentamiento y 10 s de carga con 50 conexiones, y anota
peticiones por segundo, latencias y memoria. Necesita Linux (lee la memoria de `/proc`). `oha` corre en la misma
máquina que el servicio: compara los dos entre sí, no contra mediciones de otro equipo.

## Cómo saber si quedó

```bash
./tool/comprobar_laboratorio.sh
```

Revisa formato y clippy, corre tus pruebas, levanta los dos servicios y los compara, y rompe tu código en una copia
(los tres retos de la lección) para ver que tus pruebas lo detectan.

## Requisitos

Rust 1.85 o superior, Node 22 y `curl`. Para medir, `oha`. Las versiones de `Cargo.toml` son las que probamos.

Código libre para usar y adaptar (licencia MIT).
