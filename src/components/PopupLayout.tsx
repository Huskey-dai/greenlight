import type { ReactNode } from 'react';

interface PopupLayoutProps {
  children: ReactNode;
}

export function PopupLayout({ children }: PopupLayoutProps) {
  return (
    <div className="popup-container">
      {children}
    </div>
  );
}