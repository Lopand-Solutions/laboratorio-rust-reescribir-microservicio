// Graba en casos.json lo que responde el servicio ORIGINAL a cada caso: es el contrato que Rust debe cumplir.
// Uso: node herramientas/grabar_esperado.mjs http://127.0.0.1:3001
import { readFileSync, writeFileSync } from 'node:fs';

const base = process.argv[2];
const archivo = new URL('./casos.json', import.meta.url);
const casos = JSON.parse(readFileSync(archivo));
for (const caso of casos) {
  const r = await fetch(base + caso.ruta, { method: caso.metodo });
  caso.esperado = { codigo: r.status, tipo: r.headers.get('content-type'), cuerpo: await r.text() };
}
writeFileSync(archivo, JSON.stringify(casos, null, 2) + '\n');
console.log(`${casos.length} casos grabados`);
