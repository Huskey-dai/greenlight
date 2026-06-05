#!/usr/bin/env node

/**
 * permission_prompt / notification hook: Notify Greenlight that the AI needs user input.
 *
 * Reads hook input from stdin (JSON from Claude Code).
 * Sets session state to NEEDS_INPUT with detail about the permission requested.
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

  // Extract permission description for detail
  const description = hookData.description || hookData.message || hookData.reason || '';
  const detail = description ? `Permission: ${description}` : 'Awaiting user input';

  await sendState(sessionId, 'NEEDS_INPUT', {
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
