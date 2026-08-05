import type { ReactNode } from 'react';

export function DashboardLayout({ sidebar, children }: { sidebar: ReactNode; children: ReactNode }) {
  return <div className="app-shell"><aside className="sidebar"><div className="brand"><span className="brand-mark">⌁</span><div><b>血糖戰情室</b><small>Glucose Dashboard</small></div></div>{sidebar}</aside><main className="main-content"><header className="topbar"><h1>⌂　血糖戰情室</h1><div className="top-actions"><button aria-label="主題設定">☼</button><button aria-label="使用者設定">♙　使用者⌄</button></div></header>{children}</main></div>;
}
