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

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.
