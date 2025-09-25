import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const BASE_URL = __ENV.MOP_HTTP_BASE || 'http://localhost:8000/mcp';
const API_KEY = __ENV.MOP_API_KEY || 'DEMO_KEY_PUBLIC';
const KEEP_ALIVE_WINDOW = Number(__ENV.MOP_SSE_WINDOW_MS || 5000);

export const options = {
  vus: Number(__ENV.MOP_SSE_VUS || 5),
  iterations: Number(__ENV.MOP_SSE_ITERATIONS || 20),
};

const failures = new Rate('sse_failures');
const latency = new Trend('sse_first_byte_latency', true);

function initialize() {
  const res = http.post(
    `${BASE_URL}`,
    JSON.stringify({
      jsonrpc: '2.0',
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: { name: 'k6-sse', version: '1.0.0' },
      },
      id: `sse-${Date.now()}`,
    }),
    { headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${API_KEY}` } },
  );

  const ok = check(res, { 'initialize ok': (r) => r.status === 200 && r.json('result.sessionId') });
  failures.add(ok ? 0 : 1);
  if (!ok) {
    return null;
  }

  return res.json('result.sessionId');
}

export default function () {
  const sessionId = initialize();
  if (!sessionId) {
    return;
  }

  const res = http.get(`${BASE_URL}`, {
    headers: {
      'Accept': 'text/event-stream',
      'mcp-session-id': sessionId,
      Authorization: `Bearer ${API_KEY}`,
    },
    timeout: `${KEEP_ALIVE_WINDOW}ms`,
  });

  const ok = check(res, {
    'sse status 200': (r) => r.status === 200,
    'content-type event-stream': (r) => (r.headers['Content-Type'] || '').includes('text/event-stream'),
  });

  failures.add(ok ? 0 : 1);
  if (ok) {
    latency.add(res.timings.waiting);
  }

  sleep(KEEP_ALIVE_WINDOW / 1000);
}
