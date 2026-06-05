#!/usr/bin/env node

/**
 * PostToolUse hook: Notify Greenlight that the AI is working.
 *
 * Reads hook input from stdin (JSON from Claude Code).
 * Sets session state to WORKING with an optional detail about the tool used.
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
    // No valid input, continue without detail
  }

  const sessionId = getSessionId();
  const label = getSessionLabel();

  // Extract tool name for detail
  const toolName = hookData.tool_name || hookData.tool || '';
  const detail = toolName ? `Using ${toolName}` : undefined;

  await sendState(sessionId, 'WORKING', {
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
    // Timeout after 2 seconds if no data
    setTimeout(() => resolve(data.trim()), 2000);
  });
}

// Always exit 0 - never block Claude Code.
main()
  .catch(() => {})
  .finally(() => process.exit(0));
