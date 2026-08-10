import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

/// 前端測試設定。預設環境為 node（classify.test.ts 等純邏輯測試用）；
/// 需 DOM 的元件測試在檔案頂端以 `// @vitest-environment happy-dom` 指定，
/// 避免影響既有純邏輯測試的執行速度與穩定性。
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'node',
    globals: true,
  },
});