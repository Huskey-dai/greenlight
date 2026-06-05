import { getCurrentWindow } from '@tauri-apps/api/window';
import type { MouseEvent } from 'react';
import type { DisplayState } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';

interface HeaderProps {
  aggregateState: DisplayState;
  sessionCount: number;
  diagnosticsOpen: boolean;
  onToggleDiagnostics: () => void;
}

export function Header({
  aggregateState,
  sessionCount,
  diagnosticsOpen,
  onToggleDiagnostics,
}: HeaderProps) {
  const startDrag = (event: MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    void getCurrentWindow().startDragging();
  };

  const hidePopup = () => {
    void getCurrentWindow().hide();
  };

  return (
    <div className="popup-header" data-tauri-drag-region onMouseDown={startDrag}>
      <div className="popup-header__main">
        <div className="popup-header__identity">
          <TrafficLight state={aggregateState} compact showLabel={false} />
          <span className="popup-header__title">Greenlight</span>
        </div>
        <span className="popup-header__count">
          {sessionCount === 0
            ? '\u65e0\u4f1a\u8bdd'
            : sessionCount === 1
              ? '1 \u4e2a\u4f1a\u8bdd'
              : `${sessionCount} \u4e2a\u4f1a\u8bdd`}
        </span>
      </div>
      <button
        className={`popup-header__diagnostics${diagnosticsOpen ? ' popup-header__diagnostics--active' : ''}`}
        type="button"
        aria-label="Diagnostics"
        title="Diagnostics"
        onMouseDown={(event) => event.stopPropagation()}
        onClick={onToggleDiagnostics}
      >
        ?
      </button>
      <button
        className="popup-header__close"
        type="button"
        aria-label="Close"
        title="Close"
        onMouseDown={(event) => event.stopPropagation()}
        onClick={hidePopup}
      >
        X
      </button>
    </div>
  );
}
