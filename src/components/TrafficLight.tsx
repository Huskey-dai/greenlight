import { STATE_COLORS, STATE_LABELS } from '../lib/state-constants';
import type { DisplayState } from '../lib/state-constants';
import { useReducedMotion } from '../hooks/useReducedMotion';

interface TrafficLightProps {
  state: DisplayState;
  size?: number;
  compact?: boolean;
  showLabel?: boolean;
}

export function TrafficLight({
  state,
  size = 32,
  compact = false,
  showLabel = true,
}: TrafficLightProps) {
  const prefersReducedMotion = useReducedMotion();
  const color = STATE_COLORS[state];
  const label = STATE_LABELS[state];

  const isBlinking = state === 'WORKING' || state === 'NEEDS_INPUT';
  const blinkClass =
    state === 'WORKING'
      ? 'light--working'
      : state === 'NEEDS_INPUT'
        ? 'light--needs-input'
        : '';

  // SVG rendering for high-DPI clarity
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: compact ? '6px' : '8px',
      }}
      role="img"
      aria-label={`状态: ${label}`}
    >
      <svg
        width={compact ? 10 : size}
        height={compact ? 10 : size}
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <defs>
          <radialGradient id={`glow-${state}`}>
            <stop offset="0%" stopColor={color} stopOpacity="0.3" />
            <stop offset="100%" stopColor={color} stopOpacity="0" />
          </radialGradient>
        </defs>
        {/* Outer glow */}
        <circle
          cx="12"
          cy="12"
          r="10"
          fill={`url(#glow-${state})`}
          className={isBlinking && !prefersReducedMotion ? blinkClass : undefined}
        />
        {/* Core circle */}
        <circle
          cx="12"
          cy="12"
          r="7"
          fill={color}
          className={isBlinking && !prefersReducedMotion ? blinkClass : undefined}
        />
        {/* Subtle inner light (low opacity, no white highlight) */}
        <circle
          cx="12"
          cy="10"
          r="3"
          fill="white"
          opacity="0.08"
        />
        {/* Border ring */}
        <circle
          cx="12"
          cy="12"
          r="7"
          fill="none"
          stroke="var(--light-border)"
          strokeWidth="0.5"
        />
      </svg>
      {showLabel && !compact && (
        <span style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
          {label}
        </span>
      )}
    </span>
  );
}