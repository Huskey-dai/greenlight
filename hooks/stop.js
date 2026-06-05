#!/usr/bin/env node

/**
 * Stop hook: Notify Greenlight that the session has ended.
 *
 * Sets session state to IDLE, then removes the session entirely.
 */

const { sendState, removeSession } = require('./lib/api-client');
const { getSessionId, getSessionSource } = require('./lib/session');

async function main() {
  const sessionId = getSessionId();

  // First set to IDLE
  await sendState(sessionId, 'IDLE', { source: getSessionSource() });

  // Then remove the session entirely
  await removeSession(sessionId);

  // Clean up session cache file
  const fs = require('fs');
  const path = require('path');
  const cacheFile = path.join(
    process.env.TMPDIR || process.env.TEMP || '/tmp',
    `greenlight-session-${process.pid}`
  );
  try {
    fs.unlinkSync(cacheFile);
  } catch {
    // File may not exist
  }
}

main()
  .catch(() => {})
  .finally(() => process.exit(0));
