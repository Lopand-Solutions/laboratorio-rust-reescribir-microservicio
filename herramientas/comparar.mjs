// Manda los mismos casos a los dos servicios y compara código, content-type y cuerpo, byte por byte.
// Uso: node herramientas/comparar.mjs http://127.0.0.1:3001 http://127.0.0.1:3002
import { readFileSync } from 'node:fs';

const [original, nuevo] = process.argv.slice(2);
const casos = JSON.parse(readFileSync(new URL('./casos.json', import.meta.url)));

async function pedir(base, { metodo, ruta }) {
  const r = await fetch(base + ruta, { method: metodo });
  return { codigo: r.status, tipo: r.headers.get('content-type'), cuerpo: await r.text() };
}

let diferencias = 0;
for (const caso of casos) {
  const [a, b] = await Promise.all([pedir(original, caso), pedir(nuevo, caso)]);
  const igual = a.codigo === b.codigo && a.tipo === b.tipo && a.cuerpo === b.cuerpo;
  console.log(`${igual ? '✓' : '✗'} ${caso.metodo} ${caso.ruta}`);
  if (!igual) {
    diferencias++;
    console.log(`    original: ${a.codigo} ${a.tipo} ${a.cuerpo}`);
    console.log(`    nuevo:    ${b.codigo} ${b.tipo} ${b.cuerpo}`);
  }
}
console.log(`\n${casos.length - diferencias} de ${casos.length} iguales`);
process.exit(diferencias ? 1 : 0);
