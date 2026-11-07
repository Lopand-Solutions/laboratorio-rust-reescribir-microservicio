#!/usr/bin/env bash
# Comprueba el laboratorio «Reescribe un microservicio de Node en Rust». Uso: ./tool/comprobar_laboratorio.sh
# No necesita base de datos ni red. Los retos rompen el código en una copia temporal y la borran al terminar.
set -uo pipefail
cd "$(dirname "$0")/.."
RAIZ=$(pwd)
# La copia de los retos compila en el mismo target: así no recompila todas las dependencias.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$RAIZ/target}"
export RUST_BACKTRACE=0

bien=0
mal=0
check() { # check <código de salida> <texto si pasa> <texto si falla>
  if [ "$1" -eq 0 ]; then echo "  ✓ $2"; bien=$((bien + 1)); else echo "  ✗ $3"; mal=$((mal + 1)); fi
}

echo "1. Formato y clippy"
cargo fmt --check >/dev/null 2>&1
check $? "cargo fmt --check" "cargo fmt --check encontró diferencias: corre cargo fmt"
salida=$(cargo clippy --all-targets --quiet -- -D warnings 2>&1)
check $? "clippy sin avisos" "clippy falló:
$(echo "$salida" | grep -E '^(error|warning)' | head -5)"

echo "2. Tus pruebas"
salida=$(cargo test --quiet 2>&1)
check $? "cargo test en verde ($(echo "$salida" | grep -cE '^test result: ok') suites)" \
  "cargo test falla:
$(echo "$salida" | grep -E 'panicked|FAILED|not yet implemented|  node:|  rust:' | head -8)"

echo "3. Los dos servicios, lado a lado"
if ! command -v node >/dev/null; then
  check 1 "" "no encuentro node: instálalo para comparar contra el servicio original"
else
  cargo build --quiet 2>/dev/null
  PUERTO=3101 node node/servicio.js >/dev/null 2>&1 &
  pid_node=$!
  PUERTO=3102 "$CARGO_TARGET_DIR/debug/convertidor" >/dev/null 2>&1 &
  pid_rust=$!
  for _ in $(seq 50); do
    curl -sf http://127.0.0.1:3101/salud >/dev/null && curl -sf http://127.0.0.1:3102/salud >/dev/null && break
    sleep 0.1
  done
  salida=$(node herramientas/comparar.mjs http://127.0.0.1:3101 http://127.0.0.1:3102 2>&1)
  r=$?
  kill "$pid_node" "$pid_rust" 2>/dev/null
  wait "$pid_node" "$pid_rust" 2>/dev/null
  check $r "comparar.mjs: $(echo "$salida" | tail -1)" "comparar.mjs encontró diferencias:
$(echo "$salida" | grep -A2 '^✗' | head -9)"
fi

echo "4. Tus pruebas detectan los errores de los retos (en una copia)"
# Una línea original, su versión rota y qué rompe, en el mismo orden en los tres arreglos.
ORIGINAL=(
  '    if texto.is_empty() || texto.len() > 9 || !texto.bytes().all(|b| b.is_ascii_digit()) {'
  '    let tipo = [(header::CONTENT_TYPE, "application/json; charset=utf-8")];'
  '        .find(|(k, _)| k == clave)'
)
ROTO=(
  '    if texto.is_empty() || texto.len() > 9 {'
  '    let tipo = [(header::CONTENT_TYPE, "application/json")];'
  '        .rfind(|(k, _)| k == clave)'
)
QUE=(
  'un monto «+500» que se cuela (reto 1)'
  'un content-type distinto al de Node (reto 2)'
  'que se tome el último de un parámetro repetido (reto 3)'
)
archivo=src/lib.rs
for i in "${!ORIGINAL[@]}"; do
  original=${ORIGINAL[$i]}
  if ! grep -qF -- "$original" "$archivo" 2>/dev/null; then
    check 1 "" "no encuentro en $archivo la línea «${original#"${original%%[![:space:]]*}"}»: ¿la cambiaste?"
    continue
  fi
  copia=$(mktemp -d)
  tar --exclude=./target --exclude=./.git -cf - . | tar -xf - -C "$copia"
  contenido=$(cat "$copia/$archivo")
  printf '%s\n' "${contenido/"$original"/"${ROTO[$i]}"}" >"$copia/$archivo"
  # Target aparte: si la copia rota compilara en el tuyo, dejaría su binario en target/debug.
  (cd "$copia" && CARGO_TARGET_DIR="$CARGO_TARGET_DIR/retos" cargo test --quiet >/dev/null 2>&1)
  r=$?
  rm -rf "$copia"
  [ "$r" -ne 0 ]
  check $? "detectan ${QUE[$i]}" "NO detectan ${QUE[$i]}: falta una prueba"
done

echo "5. El repositorio"
grep -qxF "target/" .gitignore 2>/dev/null
check $? ".gitignore excluye target/" ".gitignore debe tener la línea target/"

echo
if [ "$mal" -eq 0 ]; then
  echo "Laboratorio completo: $bien de $bien."
  exit 0
fi
[ "$mal" -eq 1 ] && echo "Falta 1 punto ($bien bien)." || echo "Faltan $mal puntos ($bien bien)."
exit 1
