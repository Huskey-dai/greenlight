#!/usr/bin/env node

/**
 * Stop hook: Notify Greenlight that the session has ended.
 *
 * Sets session state to IDLE, then removes the session entirely.
 */

const { sendState, removeSession } = require('./lib/api-client');
const { getSessionId, getSessionSource, clearSessionCache } = require('./lib/session');

async function main() {
  let hookData = {};

  try {
    const input = await readStdin();
    if (input) {
      hookData = JSON.parse(input);
    }
  } catch {
    // No valid input
  }

  const sessionId = getSessionId(hookData);

  // First set to IDLE
  await sendState(sessionId, 'IDLE', { source: getSessionSource() });

  // Then remove the session entirely
  await removeSession(sessionId);

  clearSessionCache(hookData);
}

function readStdin() {
  return new Promise((resolve) => {
    let data = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk) => { data += chunk; });
    process.stdin.on('end', () => resolve(data.trim()));
    process.stdin.on('error', () => resolve(''));
    setTimeout(() => resolve(data.trim()), 2000);
  });
}

main()
  .catch(() => {})
  .finally(() => process.exit(0));
