#!/usr/bin/env node

/**
 * Session identity for Greenlight hook scripts.
 *
 * Reads GREENLIGHT_SESSION_ID and GREENLIGHT_SESSION_LABEL from environment.
 * Falls back to hook-provided session identifiers, then parent-PID caching.
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const SESSION_ID_RE = /^[a-zA-Z0-9_-]{1,64}$/;

function normalizeSessionId(value) {
  const raw = String(value || '').trim();
  if (!raw) return null;
  if (SESSION_ID_RE.test(raw)) return raw;
  return crypto.createHash('sha256').update(raw).digest('hex').slice(0, 32);
}

function stableSessionKey(hookData = {}) {
  const explicit = normalizeSessionId(process.env.GREENLIGHT_SESSION_ID);
  if (explicit) return { id: explicit, cache: false };

  const candidates = [
    hookData.session_id,
    hookData.sessionId,
    hookData.conversation_id,
    hookData.conversationId,
    hookData.thread_id,
    hookData.threadId,
  ];

  for (const candidate of candidates) {
    const id = normalizeSessionId(candidate);
    if (id) return { id, cache: false };
  }

  const cacheKeySource = process.env.GREENLIGHT_SESSION_KEY || `ppid-${process.ppid}`;
  const cacheKey = crypto.createHash('sha256').update(cacheKeySource).digest('hex').slice(0, 16);
  return { id: cacheKey, cache: true };
}

function sessionCacheFile(hookData = {}) {
  const key = stableSessionKey(hookData);
  if (!key.cache) return null;

  const tempDir = process.env.TMPDIR || process.env.TEMP || '/tmp';
  return path.join(tempDir, `greenlight-session-${key.id}`);
}

function getSessionId(hookData = {}) {
  const key = stableSessionKey(hookData);
  if (!key.cache) return key.id;

  const cacheFile = path.join(
    process.env.TMPDIR || process.env.TEMP || '/tmp',
    `greenlight-session-${key.id}`
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

function clearSessionCache(hookData = {}) {
  const cacheFile = sessionCacheFile(hookData);
  if (!cacheFile) return;

  try {
    fs.unlinkSync(cacheFile);
  } catch {
    // File may not exist
  }
}

function getSessionLabel() {
  return process.env.GREENLIGHT_SESSION_LABEL || 'Session';
}

function getSessionSource() {
  const source = (process.env.GREENLIGHT_SOURCE || 'claude_code').toLowerCase();
  return source === 'codex' ? 'codex' : 'claude_code';
}

module.exports = { getSessionId, getSessionLabel, getSessionSource, clearSessionCache };
