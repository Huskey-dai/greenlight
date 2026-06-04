#!/usr/bin/env node

/**
 * Shared HTTP client for Greenlight hook scripts.
 * Posts state updates to the local Greenlight HTTP server.
 *
 * Exit code is always 0 — hook failures must never block Claude Code.
 */

const fs = require('fs');
const path = require('path');
const http = require('http');

// Read port from port.txt, fallback to 17321
function getPort() {
  const homeDir = process.env.HOME || process.env.USERPROFILE || '';
  const portFile = path.join(homeDir, '.greenlight', 'port.txt');

  try {
    const port = parseInt(fs.readFileSync(portFile, 'utf8').trim(), 10);
    if (!isNaN(port) && port > 0 && port < 65536) {
      return port;
    }
  } catch {
    // File not found or unreadable
  }
  return 17321;
}

/**
 * Send a state update to the Greenlight server.
 *
 * @param {string} sessionId - Session identifier
 * @param {string} state - One of: IDLE, WORKING, NEEDS_INPUT, ERROR
 * @param {object} options - Optional: { label, detail, source }
 * @returns {Promise<void>}
 */
async function sendState(sessionId, state, options = {}) {
  const port = getPort();
  const { label, detail, source } = options;

  const body = JSON.stringify({
    state,
    ...(label && { label }),
    ...(detail && { detail }),
    ...(source && { source }),
  });

  return new Promise((resolve) => {
    const req = http.request(
      {
        hostname: '127.0.0.1',
        port,
        path: `/api/sessions/${encodeURIComponent(sessionId)}/state`,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(body),
        },
        timeout: 3000,
      },
      (res) => {
        // Drain response to free the socket
        res.resume();
        resolve();
      }
    );

    req.on('error', () => resolve());
    req.on('timeout', () => {
      req.destroy();
      resolve();
    });

    req.write(body);
    req.end();
  });
}

/**
 * Remove a session from the Greenlight server.
 *
 * @param {string} sessionId - Session identifier
 * @returns {Promise<void>}
 */
async function removeSession(sessionId) {
  const port = getPort();

  return new Promise((resolve) => {
    const req = http.request(
      {
        hostname: '127.0.0.1',
        port,
        path: `/api/sessions/${encodeURIComponent(sessionId)}`,
        method: 'DELETE',
        timeout: 3000,
      },
      (res) => {
        res.resume();
        resolve();
      }
    );

    req.on('error', () => resolve());
    req.on('timeout', () => {
      req.destroy();
      resolve();
    });

    req.end();
  });
}

module.exports = { sendState, removeSession, getPort };