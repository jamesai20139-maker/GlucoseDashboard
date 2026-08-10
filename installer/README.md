# Windows installer

安裝目標為當前使用者的 `%LOCALAPPDATA%\GlucoseDashboard`。自策略 B（單一 exe，
前端資產以 `rust-embed` 嵌入）起，安裝只需一個 `glucose-dashboard.exe`，不再
需要連帶 `frontend/dist` 目錄。安裝會把可執行檔目錄加入使用者 `PATH`，並且
**不覆蓋**既有的設定檔 `.glucose-dashboard.json`。

## 一鍵安裝（PowerShell）

在 PowerShell 貼上並執行：

```powershell
irm https://raw.githubusercontent.com/gaistudio138/GlucoseDashboard/main/installer/get.ps1 | iex
```

腳本會：

1. 查詢 GitHub Releases 最新版本，下載 `glucose-dashboard.exe`。
2. 解壓／放置到 `%LOCALAPPDATA%\GlucoseDashboard`（保留既有設定檔）。
3. 建立 `glucose-dashboard.cmd` 啟動指令並加入使用者 `PATH`。
4. 直接啟動——後端會自動開啟瀏覽器進入 Dashboard。

之後任何**新開**的 PowerShell 視窗，直接輸入即可啟動：

```powershell
glucose-dashboard
```

> `PATH` 變更僅對新開的終端機生效；已開啟的視窗需重開。

## 僅安裝不啟動

```powershell
irm https://raw.githubusercontent.com/gaistudio138/GlucoseDashboard/main/installer/get.ps1 | iex -NoLaunch
```

> 注意：`irm ... | iex` 管道無法直接傳參數；如需 `-NoLaunch` 等參數，請先
> 下載腳本再以點號執行：
> ```powershell
> & (irm https://raw.githubusercontent.com/gaistudio138/GlucoseDashboard/main/installer/get.ps1) -NoLaunch
> ```

## 手動安裝

1. 至 [Releases](https://github.com/gaistudio138/GlucoseDashboard/releases) 下載
   `glucose-dashboard.exe`。
2. 放到 `%LOCALAPPDATA%\GlucoseDashboard`。
3. 將該目錄加入使用者 `PATH`。
4. 執行 `glucose-dashboard.exe`。

## 開發者：發布新版本

在 repo 根目錄打 tag 即觸發 CI 自動編譯並建立 Release（見
`.github/workflows/release.yml`）：

```bash
git tag v0.2.0
git push origin v0.2.0
```

CI 會以 `make build` 編譯單一 exe（含嵌入前端）並上傳為 Release 資產，
`get.ps1` 即會自動抓取此資產。