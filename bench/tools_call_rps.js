import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const BASE_URL = __ENV.MOP_HTTP_BASE || 'http://localhost:8000/mcp';
const API_KEY = __ENV.MOP_API_KEY || 'DEMO_KEY_PUBLIC';
const THINK_TIME = Number(__ENV.MOP_RPS_THINK_MS || 250) / 1000;

export const options = {
  vus: Number(__ENV.MOP_RPS_VUS || 20),
  duration: __ENV.MOP_RPS_DURATION || '1m',
  thresholds: {
    tools_call_failures: ['rate<0.05'],
  },
};

const failures = new Rate('tools_call_failures');

function initializeSession() {
  const url = `${BASE_URL}`;
  const payload = JSON.stringify({
    jsonrpc: '2.0',
    method: 'initialize',
    params: {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'k6-tools-call', version: '1.0.0' },
    },
    id: Date.now(),
  });

  const res = http.post(url, payload, {
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${API_KEY}` },
  });

  const ok = check(res, {
    'initialize succeeded': (r) => r.status === 200 && r.json('result.sessionId'),
  });
  failures.add(ok ? 0 : 1);
  if (!ok) {
    return null;
  }

  return res.json('result.sessionId');
}

export default function () {
  const sessionId = initializeSession();
  if (!sessionId) {
    sleep(THINK_TIME);
    return;
  }

  const listPayload = JSON.stringify({
    jsonrpc: '2.0',
    method: 'tools/list',
    params: {},
    id: `${Date.now()}-list`,
  });

  const listRes = http.post(BASE_URL, listPayload, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${API_KEY}`,
      'mcp-session-id': sessionId,
    },
  });

  const ok = check(listRes, {
    'tools/list ok': (r) => r.status === 200 && Array.isArray(r.json('result.tools')),
  });
  failures.add(ok ? 0 : 1);

  sleep(THINK_TIME);
}
