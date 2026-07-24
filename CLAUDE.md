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
