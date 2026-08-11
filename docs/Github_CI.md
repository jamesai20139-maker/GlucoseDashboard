# GitHub Tag → CI → Release → 一鍵安裝：原理與流程

> 本文件解釋「為什麼 `git push` 一個 tag 會自動編出可執行檔、放上 Release、
> 讓使用者用一行 `irm | iex` 就能安裝」這整條鏈是如何運作的。讀完你會理解
> 每一步對應到哪些 GitHub 機制，以及如何把它套用到其他專案。

## 一句話總覽

```
你打 tag 並 push  →  GitHub 收到 tag 事件  →  觸發 workflow
                 →  CI 編譯產物  →  上傳成 Release 資產
                 →  安裝腳本抓 Release 資產  →  使用者一鍵安裝
```

「打 tag 觸發 CI」不是魔法，是 **GitHub Actions 的事件觸發機制**；
「CI 產出的檔案變成可下載」是 **GitHub Releases + 資產（assets）機制**；
「一行指令安裝」是 **安裝腳本去呼叫 GitHub API 抓最新 Release 資產**。
三者串起來，就是這條鏈。

---

## 第 0 步：前置觀念 — Git tag 是什麼

- **commit** 是對程式碼的快照（一個雜湊，如 `a724119`）。
- **tag** 是「貼在某個 commit 上的具名標籤」，例如 `v0.2.1`。
- tag 與 branch 不同：branch 會移動，tag 永遠指著同一個 commit。
- 語意化版本（SemVer）慣例：`v0.2.1` = 主版 0、次版 2、修訂 1。
- `git push origin v0.2.1` 把 tag 上傳到遠端；**tag 不會隨 `push origin main`
  自動帶上**，必須單獨 push。

關鍵：**「push 一個 tag」是一個獨立於「push branch」的事件**，GitHub Actions
可以單獨對這個事件做反應。這就是整條鏈的起點。

---

## 第 1 步：Workflow 檔案 — 告訴 GitHub「tag 來了要做什麼」

路徑：`.github/workflows/release.yml`

```yaml
on:
  push:
    tags:
      - 'v*'
```

這段是**觸發條件（trigger）**。意思是：

- `on: push` — 當有任何 push 事件時。
- `tags: - 'v*'` — 但只限於「被 push 的東西是 tag，且名字以 `v` 開頭」。
- 所以 `git push origin main`（push branch）**不會**觸發；
  `git push origin v0.2.1`（push tag）才會。

`v*` 是萬用字元：`v0.1`、`v1.0.0`、`v2.3.4-beta` 都會 match。

> 為什麼不也對 `push main` 觸發？因為每次 push main 都編 release 太頻繁，
> 也會產生一堆無意義的「開發中版本」。只在打 tag（你決定「這是一個可發布
> 的版本點」）時才編，是發布流程的慣例。

---

## 第 2 步：CI 執行 — GitHub 幫你開一台雲端機器跑

當 tag 事件 match 到 workflow，GitHub 會：

1. 開一台 **runner**（GitHub 託管的虛擬機，本專案用 `windows-latest`）。
2. 把你的 repo 程式碼 checkout 到 runner 上。
3. 依照 workflow 的 `steps` 一步步執行。

本專案的 steps（簡化）：

```yaml
jobs:
  build-release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4        # 1. 拉程式碼
      - uses: dtolnay/rust-toolchain@stable  # 2. 裝 Rust
      - uses: actions/setup-node@v4       # 3. 裝 Node
      - run: npm --prefix frontend ci     # 4. 裝前端依賴
      - run: npm --prefix frontend run build  # 5. 編前端 → frontend/dist
      - run: cargo build --release        # 6. 編後端（rust-embed 把 dist 嵌進 exe）
      # 7. 把產物重命名為 glucose-dashboard.exe
      - uses: softprops/action-gh-release@v2  # 8. 建立 Release 並上傳資產
```

關鍵認知：**編譯發生在 GitHub 的機器上，不在使用者機器上。** 使用者永遠
只下載「已經編好的成品」，不需要裝 Rust、Node、cargo。這是「一行指令安裝」
之所以可行的根本前提 — 把編譯成本從使用者端搬到 CI 端。

---

## 第 3 步：Release 與資產 — 產物如何變成可下載

`softprops/action-gh-release@v2` 這個 step 做的事：

1. 在 repo 建立一個 **Release**，標題 = tag 名（如 `v0.2.1`）。
2. 把本地檔案 `glucose-dashboard.exe` **上傳為該 Release 的 asset（資產）**。
3. Release 頁面就是 `https://github.com/<owner>/<repo>/releases/tag/v0.2.1`。

上傳後，這個 exe 有一個**固定的下載 URL**：

```
https://github.com/<owner>/<repo>/releases/download/v0.2.1/glucose-dashboard.exe
```

這個 URL 是公開、穩定的 — 任何人（不必登入、不必裝 git）都能直接 HTTP 下載。
**這就是「使用者能一鍵拿到成品」的關鍵**：Release asset 是一個公開的檔案
下載連結，不經 git、不需認證（只要 repo 是 public）。

> 注意：Release asset 的下載 URL 是放在 GitHub 的 release 下载服務，**不是**
> `raw.githubusercontent.com`。兩者不同：raw 是「repo 某個 commit 的某個檔案」，
> release asset 是「你額外上傳到 Release 的附掛檔案」。安裝腳本抓的是後者。

---

## 第 4 步：安裝腳本 — 抓 Release 資產並安裝

`installer/get.ps1` 的核心邏輯：

```powershell
# 1. 查最新 release 的中繼資料
$apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
$release = Invoke-RestMethod -Uri $apiUrl ...

# 2. 從中繼資料裡找名為 glucose-dashboard.exe 的 asset
$asset = $release.assets | Where-Object { $_.name -eq $ExeName }

# 3. 下載該 asset
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile ...

# 4. 放到安裝目錄、建捷徑、加 PATH、啟動
```

關鍵 API：

- `GET /repos/{owner}/{repo}/releases/latest` — 回傳最新（非 prerelease）的
  Release 中繼資料 JSON，含 `assets` 陣列。
- 每個 asset 有 `browser_download_url` — 直接下載該檔案的公開 URL。

所以「一行指令」拆開來其實是：

```
irm <腳本網址>     →  PowerShell 抓回 get.ps1 的原始碼
| iex               →  立即執行該腳本
腳本內呼叫 API      →  找到最新 Release 的 exe 下載 URL
Invoke-WebRequest   →  下載 exe
Move-Item           →  放到安裝目錄
Start-Process       →  啟動
```

---

## 為什麼需要「repo 是 public」？

GitHub 對**私有 repo** 的 raw 連結與未認證 API 一律回 **404**（假裝不存在，
不回 403，避免洩漏 repo 是否存在）。所以：

- `irm https://raw.githubusercontent.com/.../get.ps1` — 私有 repo 會 404，
  使用者抓不到安裝腳本，鏈就斷在第一步。
- `GET /releases/latest` 未認證查私有 repo 也 404，腳本找不到 asset。

因此這套「公開一行指令安裝」模式**要求 repo 為 public**。私有 repo 要走
另一條（腳本內帶 token 認證），會犧牲「無腦一行」的簡單性。

---

## 完整時序圖（打 tag 那一刻起）

```
開發者本機                 GitHub 雲端                    使用者 PowerShell
─────────────              ──────────                    ──────────────────
git tag v0.2.1
git push origin v0.2.1 ──→ 收到 tag push 事件
                           match workflow trigger
                           開 windows-latest runner
                           checkout + 裝工具 + 編譯
                           產生 glucose-dashboard.exe
                           建立 Release v0.2.1
                           上傳 exe 為 asset ──────────→ (Release 頁面出現)

                                                          irm <get.ps1 網址>
                                                          ┌─ 抓回腳本
                                                          │  iex 執行
                                                          │  呼叫 /releases/latest
                                                          │  找到 asset URL
                                                          │  下載 exe
                                                          │  安裝 + 啟動
                                                          └─ Dashboard 開啟
```

整條鏈從 tag push 到使用者裝好，不需開發者再做任何手動動作 — 除了 tag push
那一行。

---

## 套用到其他專案的檢查清單

要複製這套模式到別的專案，需要：

1. **一個可編譯出單一成品的建置流程**（Rust exe、Go binary、打包成單檔的
   Electron 等）。多檔案產物需打成 zip 再上傳。
2. **`.github/workflows/release.yml`** — 觸發於 `tags: v*`，編譯、上傳 asset。
3. **`installer/get.ps1`**（或對應語言的腳本）— 呼叫 `/releases/latest`、
   下載 asset、安裝、加 PATH、啟動。
4. **repo 設為 public**（否則 raw 與 API 對外 404）。
5. **GitHub 帳號未受限制**（被 flag 的帳號即使 repo public 也對外 404）。
6. **打 tag 並 push**：`git tag v0.x.0 && git push origin v0.x.0`。

---

## 本專案的對應檔案

| 角色 | 檔案 |
|---|---|
| Workflow（觸發 + 編譯 + 上傳） | `.github/workflows/release.yml` |
| 安裝腳本（抓 asset + 安裝） | `installer/get.ps1` |
| 一行指令入口 | `README.md`、`installer/README.md` |
| 嵌入前端讓 exe 自足 | `backend/src/api/router.rs`（rust-embed） |
| Release 頁面 | `https://github.com/jamesai20139-maker/GlucoseDashboard/releases` |

## 常見疑問

- **Q：為什麼我 push 了 commit，Actions 沒跑？**
  A：commit push 到 `main` 不觸發 `release` workflow（trigger 是 `tags: v*`）。
  要 push 一個 `v` 開頭的 tag 才會跑。

- **Q：Actions 跑了但 Releases 沒出現？**
  A：檢查 workflow 的 `permissions: contents: write` 是否設定（建立 Release
  需要寫權限），以及 `softprops/action-gh-release` 的 `files:` 路徑是否正確。

- **Q：`irm | iex` 回 404？**
  A：最常見三原因 — repo 是 private、帳號被 GitHub 限制、或檔案路徑/名稱拼錯。
  用無痕視窗開 raw URL 驗證。

- **Q：安裝腳本抓到舊版？**
  A：`/releases/latest` 回傳最新的「非 prerelease」Release。若打的是 prerelease
  tag（如 `v0.2.0-beta`），不會被視為 latest。確認 tag 名與 `prerelease` 邏輯。