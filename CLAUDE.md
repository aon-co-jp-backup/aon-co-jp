# 開発方針＆開発環境ルール(aon-co-jp)

作業ドライブは`F:\open-runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。

## このリポジトリの役割(2026-07-17新設)

`aon.co.jp`のTOPページ。[`aon-tokyo`](https://github.com/aon-co-jp/aon-tokyo)
(`aon.tokyo`)と**完全に同一のソースコード・同一のコンテンツ**を配信する
(ユーザー指示: aon.tokyoとaon.co.jpは同じ内容)。技術スタック・設計方針は
`aon-tokyo`のCLAUDE.mdを参照。

`src/main.rs`は`aon-tokyo`リポジトリからのコピー。両リポジトリで機能追加・
修正を行う際は、必ず両方に同じ変更を反映すること(現状は自動同期の仕組みは
無く、手動でミラーする運用)。

## デプロイ

VPS上で`cargo build --release`、systemdサービス化、`127.0.0.1:4200`
(aon-tokyoと同じポート番号だが別ホスト/別ドメインのnginx vhostからの
リバースプロキシ先として使う想定 — 実際には同一VPS上で1つのバイナリを
2ドメイン分のvhostから指す運用が最も単純: aon.tokyo/aon.co.jp両方の
nginx vhostが同じ`127.0.0.1:4200`(aon-tokyo側のプロセス)を指せば
このリポジトリ自体を別プロセスとして起動する必要はない。将来的に
aon.tokyoとaon.co.jpの内容を分岐させたくなった場合のみ、このリポジトリを
独立した別プロセス・別ポートとして運用する)。

## HANDOFF追記(2026-07-24) — `/cancer`ページ新設(がん治療研究報道の紹介)

- 新規ルート`GET /cancer`(`cancer_page`ハンドラ)を追加し、ユーザー提供の
  実際の報道見出し・リンク2件を日英併記で紹介するページを実装した:
  (1) 「衝撃波で腫瘍を破壊「メスも針も使わない」肝臓がんの新治療法
  大阪公立大の研究チームが特定臨床研究を開始 来年中の薬事承認を目指す」
  (Facebookリンク)、(2) 「世界初！からだ自身が"がん治療" 九州大学が
  開発」(YouTubeリンク)。
- **医療情報の扱い**: ユーザー指示通り、報道見出し・リンクの紹介のみに
  徹し、独自の医療的な効能・安全性の主張や推奨は一切追加していない。
- ナビゲーションに「がん治療研究」リンクを追加(`page_shell`内`<nav>`)。
- **検証**: `cargo build`成功。実バイナリ起動+`curl http://127.0.0.1:4200/cancer`
  でHTTP 200・日英両見出し文言・両リンク先URLがHTML内に実在することを
  確認済み。
- 姉妹リポジトリ`aon-tokyo`にも同一の変更を反映済み(このリポジトリの
  既存運用ルール「両リポジトリへ同じ変更をミラーする」に従う)。

## 関連プロジェクト

- [aon-tokyo](https://github.com/aon-co-jp/aon-tokyo) — 同一コンテンツの本家
- [aruaru-tokyo-server](https://github.com/aon-co-jp/aruaru-tokyo-server) — 技術スタックの出典元

## HANDOFF追記(2026-07-24) — 無料サブドメインサービスの紹介バナー追加

- `open-easy-web`/`open-web-server`側で実装した「aon.co.jp/runo.tokyo配下
  への無料サブドメイン取得+自動更新(DDNS)機能」(`DnsProvider`トレイト、
  `ValueDomainProvider`実装)の紹介バナーをTOPページ(`/`)に追加した
  (`free_subdomain_banner()`、日英併記)。
- 寄付リンク(「ご寄付はこちらからどうぞ!」/ "Donations welcome!")は
  実リンク未準備のため`href="#"`のプレースホルダのままとし、
  「準備中 / Coming soon」と正直に表示している(実在しないURLを
  でっち上げない)。
- **検証**: `cargo build`成功、実バイナリ起動+`curl`でHTTP 200・
  バナー本文(「無料サブドメイン取得サービス」「Donations welcome」)の
  実在を確認済み。
- **正直な未検証事項**: 実Value-Domain APIキーでのサブドメイン発行・
  実GitHub OAuthでのログイン・実PostgreSQL/aruaru-dbへのデュアルライトは
  いずれも`open-web-server`側の実装であり、このリポジトリ側では未検証
  (バナー表示のみが本リポジトリの担当範囲)。

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, noch nicht von Muttersprachlern geprüft)
- **REST wird abgeschafft. Neue APIs werden nicht als REST gebaut.** APIs, die aus dem Browser oder mit `curl` aufgerufen werden, nutzen **Connect-Unary-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body); zwischen Diensten gilt **gRPC / gRPC-web** (wie bei der Admin-API von RPoem, `open_runo.admin.v1.Admin`). Streaming erst, wenn es wirklich nötig ist.
- **Bestehende REST-Endpunkte werden einzeln migriert, sobald man sie anfasst.** Nach der Migration wird die alte REST-Route entfernt. Bis dahin bleiben sie bestehen, es werden aber **keine neuen hinzugefügt**.
- **JSON wird auf RS-JSON vereinheitlicht**: Lesen mit `from_str_secure` / `parse_secure` (Grenzen für Größe, Tiefe und doppelte Schlüssel), Schreiben mit `rust_json::fast`. `prost` kommt nicht in Browser-WASM (die Datei würde größer).
- **Ausnahmen (Formate, die ein fremder Standard vorgibt, nicht unsere eigene API)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Dritten (z. B. GitHub), Health-Checks, statische Dateien, OTLP usw. Diese nehmen schlichtes HTTP entgegen, wie es der Standard verlangt.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines Repositorys diese Richtlinie schon befolgt, hängt vom jeweiligen Repository ab. Dieser Abschnitt hält die Richtlinie fest; er bedeutet nicht, dass der Code bereits geändert wurde.

### Русский (машинный перевод, носителями языка ещё не проверен)
- **REST полностью отменяется. Новые API не создаются как REST.** API, вызываемые из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело — JSON); между сервисами — **gRPC / gRPC-web** (так же, как административный API RPoem, `open_runo.admin.v1.Admin`). Потоковая передача — только когда она действительно нужна.
- **Существующие REST-эндпоинты переносятся по одному, когда до них доходят руки.** После переноса старый REST-маршрут удаляется. До завершения переноса они остаются, но **новые не добавляются**.
- **JSON унифицируется на RS-JSON**: чтение через `from_str_secure` / `parse_secure` (ограничения на размер, глубину и повторяющиеся ключи), запись через `rust_json::fast`. `prost` не добавляется в браузерный WASM (размер вырос бы).
- **Исключения (форматы, заданные чужим стандартом, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и т. п. Они принимают обычный HTTP, как требует стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Этот раздел фиксирует политику и не означает, что код уже изменён.
