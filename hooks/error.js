#!/usr/bin/env node

/**
 * Error hook: Notify Greenlight that an error has occurred.
 *
 * Reads hook input from stdin (JSON from Claude Code).
 * Sets session state to ERROR with detail about the error.
 */

const { sendState } = require('./lib/api-client');
const { getSessionId, getSessionLabel, getSessionSource } = require('./lib/session');

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

  const sessionId = getSessionId();
  const label = getSessionLabel();

  // Extract error description for detail
  const errorMsg = hookData.error || hookData.message || hookData.description || '';
  const detail = errorMsg ? String(errorMsg).slice(0, 200) : 'Error occurred';

  await sendState(sessionId, 'ERROR', {
    label,
    detail,
    source: getSessionSource(),
  });
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
