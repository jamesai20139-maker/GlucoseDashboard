# AGENTS.md

# Glucose Dashboard AI Development Guide

本專案採用 Spec Kit 開發流程。

所有 AI Agent 在開始任何工作前，都必須遵守本文件。

---

# 工作流程

開始任何 Task 前：

1. 閱讀 PRD.md
2. 閱讀目前 Feature 的 spec.md
3. 閱讀 plan.md
4. 閱讀 data-model.md（若存在）
5. 閱讀 contracts（若存在）
6. 閱讀 tasks.md

不得直接開始 Coding。

---

# 開發原則

採用：

Documentation First

Architecture First

Small Increment

One Task at a Time

---

# Coding 規範

一次只完成 tasks.md 的一個 Task。

不得自行開始下一個 Task。

若目前 Task 完成：

- 執行測試
- 更新文件
- 等待 Product Owner Review

---

# 修改限制

不得：

- 修改已完成 Feature 的需求
- 修改 spec.md
- 修改 plan.md

若需求有問題：

提出 Issue。

不得自行修改。

---

# 測試

所有 Task 都必須：

- 撰寫 Unit Test
- 執行全部測試
- 修正 Warning

不得略過測試。

---

# 文件

若程式修改影響文件：

同步更新：

- README
- PRD（若需求改變）
- Spec Kit 文件（若 Product Owner 同意）

---

# Git

每完成一個 Task：

建議 Git Commit Message,並且使用繁體中文

不得自動 Push。

---

# 停止條件

完成目前 Task：

立即停止。

等待 Product Owner Review。

不得自行開始下一個 Task。

---

# 專案技術

Backend：

Rust

Frontend：

React

UI：

Browser

Data Source：

Google Sheet

Architecture：

Google Sheet

↓

Rust Backend

↓

REST API

↓

React

↓

Browser