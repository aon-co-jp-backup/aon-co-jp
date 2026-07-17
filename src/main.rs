//! aon.tokyo / aon.co.jp — Rust + Poem 版TOPページ。
//! aruaru-tokyo-server (aruaru.tokyo) と同じ技術スタック・実装方針を踏襲する:
//! DB非依存・1バイナリ完結・サーバーサイド文字列組み立てHTML(テンプレート
//! エンジン不使用)。aon.tokyo と aon.co.jp は同一バイナリ・同一コンテンツを
//! 配信する(nginx側で両ドメインを同じ127.0.0.1:ポートへリバースプロキシする
//! 運用、aruaru-tokyo-server と同じ配置パターン)。
//!
//! テーマ: AI・IT・WEB・オーディオ(JBL・B&W等の大型スピーカー含む)。
//! aruaru.tokyoの既存TOPページとは相互リンクで連携する(統合)。
//!
//! ## 「クリックで検索」リンクの方針(2026-07-17、ユーザー指示)
//! 検索結果の長いURL(トラッキングパラメータだらけのGoogle/YouTube検索結果
//! URL)をそのままページに貼らない。代わりに、検索エンジン自身の
//! `?q=`/`?search_query=`形式の短いURLを組み立て、クリックした瞬間に
//! ブラウザ側でその都度検索・表示させる(`search_link`/`youtube_search_link`)。

use poem::listener::TcpListener;
use poem::web::Html;
use poem::{get, handler, Route, Server};

const ARUARU_TOKYO_URL: &str = "https://aruaru.tokyo/";
const GITHUB_ORG_URL: &str = "https://github.com/aon-co-jp";

/// 手書きのpercent-encoding(RFC 3986のquery成分に必要な最小限の置換のみ)。
/// 外部crateへ依存させない、というこのエコシステムの既存方針に合わせる。
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

/// Google検索結果への「クリックで検索」リンク(短いクエリのみをURLに埋め込み、
/// 長いトラッキング付き結果URLは一切貼らない)。
fn google_search_link(label: &str, query: &str) -> String {
    format!(
        r#"<a href="https://www.google.com/search?q={}" target="_blank" rel="noopener noreferrer">🔎 {}</a>"#,
        percent_encode(query),
        label
    )
}

/// YouTube検索結果への同様のリンク。
fn youtube_search_link(label: &str, query: &str) -> String {
    format!(
        r#"<a href="https://www.youtube.com/results?search_query={}" target="_blank" rel="noopener noreferrer">▶️ {}</a>"#,
        percent_encode(query),
        label
    )
}

fn page_shell(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
body {{ font-family: -apple-system, "Hiragino Sans", "Yu Gothic", sans-serif; max-width: 780px; margin: 2rem auto; padding: 0 1rem; line-height: 1.7; color: #222; }}
h1 {{ font-size: 1.6rem; }}
h2 {{ font-size: 1.2rem; margin-top: 2rem; border-bottom: 2px solid #eee; padding-bottom: 0.3rem; }}
a {{ color: #222; }}
a:visited {{ color: #222; }}
nav a {{ margin-right: 1rem; }}
ul.linklist li {{ margin-bottom: 0.5rem; }}
footer {{ margin-top: 3rem; font-size: 0.85rem; color: #777; }}
</style>
</head>
<body>
<nav><a href="/">TOP</a> <a href="/links">リンク集</a> <a href="/municipal">地域・企業誘致提案</a> <a href="{ARUARU_TOKYO_URL}">aruaru.tokyo</a></nav>
{body}
<footer><p>このサイトは aon.tokyo / aon.co.jp として同一内容を配信しています。 <a href="{GITHUB_ORG_URL}">GitHub (aon-co-jp)</a></p></footer>
</body>
</html>"#
    )
}

#[handler]
fn healthz() -> &'static str {
    "ok"
}

#[handler]
fn top() -> Html<String> {
    let audio_brands = ["JBL", "B&W (Bowers & Wilkins)", "YAMAHA NS-1000", "Klipsch", "Tannoy"];
    let audio_links: String = audio_brands
        .iter()
        .map(|b| format!("<li>{}</li>", youtube_search_link(b, &format!("{b} 大型スピーカー レビュー"))))
        .collect();

    let body = format!(
        r#"<h1>aon.tokyo / aon.co.jp</h1>
<p>AI・IT・WEB開発、そして本格オーディオ(JBL・B&amp;Wなどの大型スピーカー・アンプ)を扱うサイトです。
aon.tokyo と aon.co.jp は同一内容を配信しています。 姉妹サイト <a href="{ARUARU_TOKYO_URL}">aruaru.tokyo</a> と相互に連携しています。</p>

<h2>AI・IT・WEB</h2>
<ul class="linklist">
<li>{ai_it_web}</li>
<li>{ai_animation}</li>
</ul>

<h2>オーディオ(AUDIO)</h2>
<ul class="linklist">
{audio_links}
</ul>
"#,
        ai_it_web = google_search_link("AI・IT・WEB開発の最新動向を検索", "AI IT WEB開発 最新動向"),
        ai_animation = google_search_link("AIでアニメーション作成している企業のホームページを検索", "AI企業 AIでアニメーション作成 している ホームページ"),
    );
    Html(page_shell("aon.tokyo / aon.co.jp — AI・IT・WEB・AUDIO", &body))
}

#[handler]
fn links_page() -> Html<String> {
    let items = [
        google_search_link("コミケのアニソン、フランスなど外国のYouTube検索結果", "コミケ アニソン フランス 海外 YouTube"),
        google_search_link("一級建築士・専攻建築士(建築管理士) ベテラン 75歳 80歳 再雇用", "一級建築士 専攻建築士 建築管理士 ベテラン 75歳 80歳 再雇用"),
        google_search_link("ヤマダホーム・パナホームのコーキングレス/シーリングレス外壁", "ヤマダホーム パナホーム コーキングレス シーリングレス 外壁"),
        google_search_link("スマホなど産業廃棄物の灰から取れるレアアースの種類", "スマホ 産業廃棄物 灰 レアアース 種類"),
        google_search_link("日本の海底の泥からのレアアース採取(南鳥島沖)", "日本 海底 泥 レアアース 南鳥島"),
        google_search_link("GitHubリポジトリの自動バックアップ(Windows/NAS)", "GitHub リポジトリ 自動バックアップ Windows NAS"),
        google_search_link("無料で未経験からIT研修と無料の転職エージェントサービス", "無料 未経験 IT研修 無料 転職エージェントサービス"),
        google_search_link("AI企業 AIでアニメーション作成しているホームページ", "AI企業 AIでアニメーション作成 している ホームページ"),
    ];
    let list: String = items.iter().map(|i| format!("<li>{i}</li>")).collect();

    // KAZUMA (言語交換/多言語コミュニケーション動画) — 動画そのものを埋め込まず、
    // 検索結果ではなく実際に提供されたリンク先へ直接リンクする(既存URL、生成URLではない)。
    let kazuma_links = r#"
<li><a href="https://youtube.com/shorts/Eyh9uyuQ8ug?si=VBpwkEsbXtWmBQV6" target="_blank" rel="noopener noreferrer">▶️ KAZUMA 世界中とTVチャット (YouTube Shorts)</a></li>
<li><a href="https://www.facebook.com/reel/1610434530792489?locale=ja_JP" target="_blank" rel="noopener noreferrer">📘 同上 (Facebook Reel)</a></li>
<li><a href="https://youtube.com/shorts/OJkd7CEDTtk" target="_blank" rel="noopener noreferrer">▶️ 言語を話して「退屈」から目が輝き出す瞬間 (YouTube Shorts)</a></li>
<li><a href="https://www.facebook.com/share/v/18s1gwqZa3/" target="_blank" rel="noopener noreferrer">📘 同上 (Facebook)</a></li>
<li><a href="https://youtube.com/shorts/pmQgc7j6xxE?si=pHLo961bsjEMvQqg" target="_blank" rel="noopener noreferrer">▶️ 日本人がポーランド語で話しかけたら (YouTube Shorts)</a></li>
<li><a href="https://www.facebook.com/reel/2212562739578992/?app=fbl" target="_blank" rel="noopener noreferrer">📘 同上 (Facebook Reel)</a></li>
"#;

    let body = format!(
        r#"<h1>リンク集</h1>
<p>検索結果は長いURLのまま貼らず、クリックした瞬間にその都度検索する形式にしています。</p>
<ul class="linklist">
{list}
</ul>
<h2>KAZUMA — 多言語コミュニケーション動画</h2>
<ul class="linklist">
{kazuma_links}
</ul>
<h2>参考リンク(fabeee)</h2>
<ul class="linklist">
<li><a href="https://fabeee.co.jp/business/" target="_blank" rel="noopener noreferrer">fabeee — 無料IT研修・転職エージェント事業紹介</a></li>
</ul>
"#
    );
    Html(page_shell("リンク集 | aon.tokyo", &body))
}

#[handler]
fn municipal_page() -> Html<String> {
    let cities = ["あきる野市", "旧五日市町", "青梅市", "奥多摩町", "昭島市"];
    let city_links: String = cities
        .iter()
        .map(|c| format!("<li>{}</li>", google_search_link(&format!("{c} 役所 ホームページ"), &format!("{c} 役所 ホームページ"))))
        .collect();

    let body = format!(
        r#"<h1>地域・企業誘致提案 (あきる野市・青梅市・奥多摩町・昭島市 周辺)</h1>
<p>地方・郊外でも大きな工場や倉庫の誘致を推進するための提案ページです。
テレワーク/リモートワーク推進、農業・林業・陸上養殖(魚介類)の普及、
ドローン空撮による工場・倉庫・企業誘致PRなどをまとめています。</p>

<p><strong>注記:</strong> 個人が特定できる第三者(親子等)の写真をGoogle画像検索等から
収集して掲載することは、プライバシー・著作権の観点から本ページには含めていません。</p>

<h2>関連自治体ホームページ</h2>
<ul class="linklist">
{city_links}
</ul>

<h2>ドローン空撮・企業誘致PR</h2>
<ul class="linklist">
<li>{drone}</li>
</ul>

<h2>ごみ・廃棄物の再資源化</h2>
<ul class="linklist">
<li>{fermentation}</li>
<li>{tunnel_compost}</li>
<li>{mercury_filter}</li>
<li>{mixed_fuel}</li>
<li>{plastic_oil}</li>
<li>{orange_oil}</li>
</ul>

<h2>陸上養殖・農業・林業</h2>
<ul class="linklist">
<li>{aquaculture}</li>
<li>{aquaculture_success_g}</li>
<li>{aquaculture_success_yt}</li>
<li>{aquaculture_failure_g}</li>
<li>{aquaculture_failure_yt}</li>
</ul>
"#,
        drone = youtube_search_link("工場・倉庫誘致PR ドローン空撮", "工場 倉庫 企業誘致 ドローン 空撮"),
        fermentation = youtube_search_link("生ごみを発酵させて肥料・燃料にする方法", "生ごみ 発酵 肥料 燃料 納豆菌"),
        tunnel_compost = youtube_search_link("生ごみ+燃えるごみを一緒に発酵させるトンネルコンポスト方式", "トンネルコンポスト 生ごみ 燃えるごみ 発酵 肥料 燃料"),
        mercury_filter = google_search_link("ごみ焼却時の水銀ガス回収 日立造船フィルター", "ごみ焼却 水銀ガス 回収 日立造船 フィルター"),
        mixed_fuel = youtube_search_link("燃えるごみとプラスチックごみを混ぜて燃料にする", "燃えるごみ プラスチックごみ 混ぜる 燃料"),
        plastic_oil = youtube_search_link("廃プラスチックを業務用マイクロ波で石油にする", "廃プラスチック 業務用 マイクロ波 石油"),
        orange_oil = youtube_search_link("発泡スチロールをオレンジオイルで溶かす", "発泡スチロール オレンジオイル 溶かす"),
        aquaculture = youtube_search_link("陸上養殖(プール・水槽)魚介類 普及", "陸上養殖 プール 水槽 魚介類"),
        aquaculture_success_g = google_search_link("陸上養殖 成功例", "陸上養殖 成功例"),
        aquaculture_success_yt = youtube_search_link("陸上養殖 成功例", "陸上養殖 成功例"),
        aquaculture_failure_g = google_search_link("陸上養殖 失敗例", "陸上養殖 失敗例"),
        aquaculture_failure_yt = youtube_search_link("陸上養殖 失敗例", "陸上養殖 失敗例"),
    );
    Html(page_shell("地域・企業誘致提案 | aon.tokyo", &body))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();
    let app = Route::new()
        .at("/", get(top))
        .at("/healthz", get(healthz))
        .at("/links", get(links_page))
        .at("/municipal", get(municipal_page));

    tracing::info!("aon-tokyo-server listening on 127.0.0.1:4200");
    Server::new(TcpListener::bind("127.0.0.1:4200")).run(app).await
}
