#!/usr/bin/env node

/**
 * Session identity for Greenlight hook scripts.
 *
 * Reads GREENLIGHT_SESSION_ID and GREENLIGHT_SESSION_LABEL from environment.
 * Falls back to UUID v4 generation with PID-based caching.
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// Cache session ID in a temp file keyed by PID
function getSessionId() {
  // Prefer environment variable
  if (process.env.GREENLIGHT_SESSION_ID) {
    return process.env.GREENLIGHT_SESSION_ID;
  }

  // Try to read cached session ID
  const cacheFile = path.join(
    process.env.TMPDIR || process.env.TEMP || '/tmp',
    `greenlight-session-${process.pid}`
  );

  try {
    const cached = fs.readFileSync(cacheFile, 'utf8').trim();
    if (cached) return cached;
  } catch {
    // No cached file
  }

  // Generate new UUID v4
  const newId = crypto.randomUUID();
  try {
    fs.writeFileSync(cacheFile, newId, 'utf8');
  } catch {
    // Cannot write cache, continue anyway
  }

  return newId;
}

function getSessionLabel() {
  return process.env.GREENLIGHT_SESSION_LABEL || 'Session';
}

function getSessionSource() {
  const source = (process.env.GREENLIGHT_SOURCE || 'claude_code').toLowerCase();
  return source === 'codex' ? 'codex' : 'claude_code';
}

module.exports = { getSessionId, getSessionLabel, getSessionSource };
