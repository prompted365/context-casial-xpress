import ws from 'k6/ws';
import { sleep } from 'k6';
import { Rate } from 'k6/metrics';

const BASE_URL = __ENV.MOP_BASE_URL || 'ws://localhost:8000/ws';
const SESSION_SLEEP_MS = Number(__ENV.MOP_WS_SLEEP_MS || 500);

export const options = {
  vus: Number(__ENV.MOP_WS_VUS || 50),
  duration: __ENV.MOP_WS_DURATION || '1m',
  thresholds: {
    ws_failures: ['rate<0.05'],
  },
};

const failures = new Rate('ws_failures');

export default function () {
  const params = {};
  const url = BASE_URL;

  let hadError = false;

  ws.connect(url, params, function (socket) {
    socket.on('open', function () {
      // minimal ping to keep the connection alive
      socket.send(JSON.stringify({ jsonrpc: '2.0', method: 'ping', id: `${Date.now()}-${Math.random()}` }));
    });

    socket.on('message', function () {
      // ignore payloads; this test focuses on connection churn
    });

    socket.on('error', function (e) {
      hadError = true;
    });

    socket.on('close', function () {
      failures.add(hadError ? 1 : 0);
    });

    sleep(SESSION_SLEEP_MS / 1000);
    socket.close();
  });
}
