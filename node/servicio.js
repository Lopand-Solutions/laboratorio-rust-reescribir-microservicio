// El microservicio original, en Node sin dependencias: convierte montos entre monedas con una tabla fija.
// Las tasas son de ejemplo, no reales. Todo va en enteros (centavos y millonésimas) para no depender de decimales.
import { createServer } from 'node:http';

const TASAS = {
  'USD-MXN': 18500000, // 1 USD = 18.5 MXN
  'MXN-USD': 54054,
  'EUR-MXN': 20100000,
  'MXN-EUR': 49751,
  'USD-EUR': 920000,
  'EUR-USD': 1086957,
};
// Con montos hasta 100,000,000 centavos y tasas menores a 30,000,000, el producto cabe en
// Number.MAX_SAFE_INTEGER (9e15): por arriba de eso, Number perdería precisión.
const MONTO_MAXIMO = 100000000;

function convertir(params) {
  const de = params.get('de');
  const a = params.get('a');
  const monto = params.get('monto');
  if (!de || !a || monto === null) {
    return [400, { error: 'faltan de, a o monto' }];
  }
  if (!/^\d{1,9}$/.test(monto) || Number(monto) < 1 || Number(monto) > MONTO_MAXIMO) {
    return [400, { error: `monto debe ser un entero de centavos entre 1 y ${MONTO_MAXIMO}` }];
  }
  const tasa = TASAS[`${de}-${a}`];
  if (tasa === undefined) {
    return [404, { error: `no hay tasa de ${de} a ${a}` }];
  }
  const centavos = Number(monto);
  return [200, {
    de,
    a,
    monto_centavos: centavos,
    tasa_millonesimas: tasa,
    // Redondeo a la mitad hacia arriba, en enteros.
    resultado_centavos: Math.floor((centavos * tasa + 500000) / 1000000),
  }];
}

function responder(res, codigo, cuerpo) {
  res.writeHead(codigo, { 'content-type': 'application/json; charset=utf-8' });
  res.end(JSON.stringify(cuerpo));
}

const RUTAS = {
  '/salud': () => [200, { estado: 'ok' }],
  '/convertir': convertir,
};

const servidor = createServer((req, res) => {
  const url = new URL(req.url, 'http://localhost');
  const ruta = RUTAS[url.pathname];
  if (!ruta) return responder(res, 404, { error: 'no encontrado' });
  if (req.method !== 'GET') return responder(res, 405, { error: 'método no permitido' });
  const [codigo, cuerpo] = ruta(url.searchParams);
  responder(res, codigo, cuerpo);
});

const puerto = Number(process.env.PUERTO || 3001);
servidor.listen(puerto, '0.0.0.0', () => console.log(`node escuchando en ${puerto}`));
process.on('SIGTERM', () => servidor.close(() => process.exit(0)));
process.on('SIGINT', () => servidor.close(() => process.exit(0)));
