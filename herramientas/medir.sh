#!/usr/bin/env bash
# Mide los dos servicios con la misma carga, uno después del otro. Uso: ./herramientas/medir.sh [segundos] [conexiones]
# Necesita Linux (lee la memoria de /proc), node, curl y oha (cargo install oha).
# oha y el servicio corren en la misma máquina y se pelean los núcleos: compara entre sí, no con otras mediciones.
set -euo pipefail
cd "$(dirname "$0")/.."
SEG=${1:-10}
CON=${2:-50}
RUTA='/convertir?de=USD&a=MXN&monto=10000'
command -v oha >/dev/null || { echo "falta oha: cargo install oha"; exit 1; }

cargo build --release --quiet
BIN="${CARGO_TARGET_DIR:-target}/release/convertidor"

echo "Equipo: $(nproc) núcleos, $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs), $(free -h | awk '/Mem:/{print $2}') de RAM"
echo "Versiones: node $(node -v), $(rustc -V | cut -d' ' -f1-2), $(oha --version)"
echo "Carga: ${SEG} s con ${CON} conexiones a $RUTA, después de 2 s de calentamiento"
echo
echo "| Servicio | Peticiones/s | p50 (ms) | p99 (ms) | Éxito | RAM en reposo (MB) | RAM pico (MB) |"
echo "| --- | --- | --- | --- | --- | --- | --- |"

medir() { # medir <nombre> <puerto> <comando…>
  local nombre=$1 puerto=$2
  shift 2
  PUERTO=$puerto "$@" >/dev/null 2>&1 &
  local pid=$!
  for _ in $(seq 50); do
    curl -sf "http://127.0.0.1:$puerto/salud" >/dev/null && break
    sleep 0.1
  done
  local reposo pico json
  reposo=$(awk '/VmRSS/ {print $2}' "/proc/$pid/status")
  oha -z 2s -c "$CON" --no-tui --output-format quiet "http://127.0.0.1:$puerto$RUTA"
  json=$(oha -z "${SEG}s" -c "$CON" --no-tui --output-format json "http://127.0.0.1:$puerto$RUTA")
  pico=$(awk '/VmHWM/ {print $2}' "/proc/$pid/status")
  kill -INT "$pid"
  wait "$pid" 2>/dev/null || true
  node -e '
    const [json, nombre, reposo, pico] = process.argv.slice(1);
    const d = JSON.parse(json);
    const ms = (s) => (s * 1000).toFixed(2);
    const mb = (kb) => (kb / 1024).toFixed(1);
    console.log(`| ${nombre} | ${Math.round(d.summary.requestsPerSec)} | ${ms(d.latencyPercentiles.p50)} | ` +
      `${ms(d.latencyPercentiles.p99)} | ${(d.summary.successRate * 100).toFixed(1)} % | ${mb(reposo)} | ${mb(pico)} |`);
  ' "$json" "$nombre" "$reposo" "$pico"
}

medir "Node (node:http)" 3001 node node/servicio.js
medir "Rust (Axum, release)" 3002 "$BIN"
