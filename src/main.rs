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

/// Google画像検索結果への同様のリンク。
fn google_image_search_link(label: &str, query: &str) -> String {
    format!(
        r#"<a href="https://www.google.com/search?tbm=isch&q={}" target="_blank" rel="noopener noreferrer">🖼️ {}</a>"#,
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

/// Googleの動画検索(YouTube以外の動画サイトも横断的に含まれる)を、
/// クリックした瞬間に行うリンク。
fn google_video_search_link(label: &str, query: &str) -> String {
    format!(
        r#"<a href="https://www.google.com/search?q={}&tbm=vid" target="_blank" rel="noopener noreferrer">🎬 {}</a>"#,
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
<nav><a href="/">TOP</a> <a href="/links">リンク集</a> <a href="/municipal">地域・企業誘致提案</a> <a href="/cancer">がん治療研究</a> <a href="{ARUARU_TOKYO_URL}">aruaru.tokyo</a></nav>
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

/// ページ最上部に置く、DuckDNS対応の簡単ドメイン設定の一文紹介
/// (日英併記、ユーザー指示原文を採用、2026-07-24追加)。
fn easy_free_domain_top_banner() -> String {
    r##"<p style="border:1px solid #ddd;border-radius:8px;padding:0.75rem 1rem;background:#f9f9f9;">
<strong>Easy Free-Domain Setup, DuckDNS(簡単ドメイン設定)</strong> —
固定IPではないDDNS環境向けに、無料サブドメイン(DuckDNS)の取得〜自動更新を
open-web-server側で一気通貫にセットアップします。複数のドメイン提供で、
サブドメイン取得可能です。<br>
<span style="color:#555;">For dynamic-IP (non-fixed-IP) environments, get a
free DuckDNS subdomain and set up automatic updates end-to-end via
open-web-server. Multiple base domains are offered to choose your subdomain
from.</span></p>"##
        .to_string()
}

/// 「無料サブドメイン取得+自動更新」機能の紹介バナー(日英併記、
/// 2026-07-24追加)。DuckDNSのような無料サブドメイン発行サービスを、
/// aon.co.jp自身のドメイン配下で提供する(`open-easy-web`/`open-web-server`
/// 側の`DnsProvider`実装、詳細はそちら側のCLAUDE.md参照)。
/// **寄付リンクは未準備のためプレースホルダ(`#`)のまま、「準備中」である
/// 旨を正直に表示する**(実在しないURLをでっち上げない、ユーザー指示)。
fn free_subdomain_banner() -> String {
    r##"<section style="border:1px solid #ddd;border-radius:8px;padding:1rem;margin:1.5rem 0;background:#f9f9f9;">
<h2 style="margin-top:0;border-bottom:none;">🌐 無料サブドメイン取得サービス / Free Subdomain Service</h2>
<p>aon.co.jp配下(および複数の提供ドメインから選択可能)のサブドメインを
無料で取得し、固定IPが無くても自動的に最新のIPアドレスへ更新し続けられる
機能を提供しています(DuckDNSのような無料DDNSサービスと同種の仕組みです)。
詳しくは
<a href="https://github.com/aon-co-jp/open-easy-web" target="_blank" rel="noopener noreferrer">open-easy-web</a>
をご覧ください。</p>
<p style="color:#555;">Get a free subdomain under aon.co.jp and keep it automatically pointed at your
current IP address, even without a fixed IP — similar to free DDNS services like DuckDNS.
See <a href="https://github.com/aon-co-jp/open-easy-web" target="_blank" rel="noopener noreferrer">open-easy-web</a> for details.</p>
<p><strong>ご寄付はこちらからどうぞ!</strong>(<a href="#" title="寄付リンクは準備中です">準備中 / Coming soon</a>) /
<strong>Donations welcome!</strong> (<a href="#" title="Donation link is not ready yet">link coming soon</a>)</p>
</section>"##.to_string()
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

{easy_free_domain_top_banner}

<p>AI・IT・WEB開発、そして本格オーディオ(JBL・B&amp;Wなどの大型スピーカー・アンプ)を扱うサイトです。
aon.tokyo と aon.co.jp は同一内容を配信しています。 姉妹サイト <a href="{ARUARU_TOKYO_URL}">aruaru.tokyo</a> と相互に連携しています。</p>

{free_subdomain_banner}

<h2>AI・IT・WEB</h2>
<ul class="linklist">
<li>{ai_it_web}</li>
<li>{ai_animation}</li>
</ul>

<h2>オーディオ(AUDIO)</h2>
<ul class="linklist">
{audio_links}
</ul>

<h3>アンプメーカー</h3>
<ul class="linklist">
<li>{accuphase_site}</li>
<li>{accuphase_yt}</li>
<li>{luxman_site}</li>
<li>{luxman_yt}</li>
<li>{yamaha_amp_site}</li>
<li>{yamaha_amp_yt}</li>
</ul>

<h3>SPEC RPA-MG1000 / RPA-MG3000(国産最高峰の超弩級ハイエンドアンプ)</h3>
<p>日本の高級オーディオメーカー、スペック株式会社(SPEC)のフラグシップ・パワーアンプ。
アンプ本体と巨大な外付け電源ユニットが同一サイズのウッドパネル筐体で横並びになる特徴的なスタイルで、
モノラルペア(計4筐体)で税込約1,000万円に達する。</p>
<ul class="linklist">
<li>{spec_photo}</li>
<li>{spec_yt}</li>
</ul>

<h3>PASS LABS</h3>
<ul class="linklist">
<li><a href="https://www.moon-audio.com/collections/brands-pass-labs" target="_blank" rel="noopener noreferrer">Pass Labs (English, via Moon Audio)</a></li>
<li><a href="https://www.electori.co.jp/pass/" target="_blank" rel="noopener noreferrer">Pass Labs (日本正規代理店、株式会社エレクトリ)</a></li>
</ul>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">想像を超えた宇宙の広さ、地球のちっぽけさを体感してください。<br><span style="color:#555;">Experience the vastness of space beyond imagination, and how tiny Earth truly is.</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/HXwtnUEga7M" title="想像を超えた宇宙の広さ、地球のちっぽけさを体感してください。 / Experience the vastness of space beyond imagination, and how tiny Earth truly is." frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: #555;">動画が表示されない場合はこちらで検索 / If the video does not play, search here: {space_video_search}</p>
<p style="font-size: 0.85rem; color: #555;">YouTubeのリンクが切れたらYouTube検索と次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via YouTube search or the following URL:
<a href="https://www.facebook.com/reel/1514589143178165?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<h2>世界平和・環境保護・貿易への願い / A Wish for World Peace, Environmental Care & Trade</h2>
<p>
戦争や兵器の高度化よりも、世界中が地球の自然環境を大切にしながら、
貿易を通じて共に経済発展していくこと——それが、このサイトを運営する
個人としての願いです。国と国が兵器で争うのではなく、地球という
一つの星を守りながら豊かになっていく道を、これからも大切に考えて
いきたいと思っています。国境を越えた協力によって世界の平和と地球
環境の保護が進んでいくことを願う、一個人としてのビジョンを紹介する
ものであり、特定の実在組織の公式な活動内容や見解を代表するものでは
ありません。
</p>
<p>
自然環境の保護と経済発展は対立するものではなく、両立できるものだと
考えています。日本とアメリカ、そして世界の国々が、貿易や文化交流を
通じて結びつきながら、地球の自然環境保護にもつながっていくことを
願っています。オンライン不動産や建設・工務店のSET(組み合わせ)事業を
はじめ、国内外の航空機・自動車・バイク、オーディオのアンプ・
スピーカー・DDC・DAC、家電製品、食料品、雑貨、医療品など、世界中との
貿易・交易が、国益にも資し、世界中の経済発展につながっていくことを、
心から願っております。スマホ・タブレット・PC・グラフィックボードや
モニターなど情報機器の周辺機器といったハードウェアの売買やレンタル・
リース、オンラインのTVチャットなども貿易の対象として想定しております。
また、SAPのカスタマイズ業務の特別注文(スペシャルオーダー)やWEBアプリ
開発などの特別注文(スペシャルオーダー)も受け付けております。是非
いつでもご注文くださいませ!
</p>
<p style="color:#555;">
Rather than war or ever more advanced weapons, I personally wish for a
world where every nation values and protects the Earth's natural
environment while growing together economically through trade. This
site shares one individual's hope: that nations can prosper by caring
for our shared planet, rather than through conflict. This introduces a
personal vision — a hope that cooperation across borders will advance
both world peace and the protection of the global environment — and
does not represent the official activities or views of any real
organization.
</p>
<p style="color:#555;">
Environmental protection and economic development need not be at odds.
I hope that Japan, the United States, and nations around the world can
grow closer through trade and cultural exchange, in ways that also
support the protection of our planet's natural environment. From
online real estate and combined construction/contractor ("SET")
businesses, to aircraft, automobiles, motorcycles, audio amplifiers,
speakers, DDCs and DACs, home appliances, food, everyday goods, and
medical supplies — I sincerely wish that trade with the whole world, in
all these areas, benefits every nation's interests and contributes to
economic development worldwide. This also extends to buying, selling,
renting, and leasing hardware such as smartphones, tablets, PCs,
graphics cards, and monitors and other IT peripherals, as well as
online video chat services. Custom/special orders for SAP customization
work and web app development are also welcome — please feel free to
reach out anytime!
</p>
"#,
        ai_it_web = google_search_link("AI・IT・WEB開発の最新動向を検索", "AI IT WEB開発 最新動向"),
        ai_animation = google_search_link("AIでアニメーション作成している企業のホームページを検索", "AI企業 AIでアニメーション作成 している ホームページ"),
        accuphase_site = google_search_link("Accuphase 公式サイト", "Accuphase アキュフェーズ 公式サイト"),
        accuphase_yt = youtube_search_link("Accuphase アンプ レビュー", "Accuphase アキュフェーズ アンプ レビュー"),
        luxman_site = google_search_link("LUXMAN 公式サイト", "LUXMAN ラックスマン 公式サイト"),
        luxman_yt = youtube_search_link("LUXMAN アンプ レビュー", "LUXMAN ラックスマン アンプ レビュー"),
        yamaha_amp_site = google_search_link("YAMAHA アンプ 公式サイト", "YAMAHA アンプ 公式サイト"),
        yamaha_amp_yt = youtube_search_link("YAMAHA アンプ スピーカー レビュー", "YAMAHA アンプ スピーカー レビュー"),
        spec_photo = google_image_search_link("SPEC RPA-MG1000 RPA-MG3000(画像)", "SPEC RPA-MG1000 RPA-MG3000"),
        spec_yt = youtube_search_link("SPEC RPA-MG1000 RPA-MG3000", "SPEC RPA-MG1000 RPA-MG3000"),
        space_video_search = youtube_search_link(
            "想像を超えた宇宙の広さ、地球のちっぽけさを体感してください。 / Experience the vastness of space beyond imagination, and how tiny Earth truly is.",
            "想像を超えた宇宙の広さ、地球のちっぽけさを体感してください。 / Experience the vastness of space beyond imagination, and how tiny Earth truly is."
        ),
        free_subdomain_banner = free_subdomain_banner(),
        easy_free_domain_top_banner = easy_free_domain_top_banner(),
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

<h2>Google Chromeブラウザで「保護されていない通信」と出る場合の対処方法</h2>
<p>Edge(Windowsの証明書ストアを使用)では正常なのに対し、Chromeは独自の
「Chrome Root Store」という、Windowsとは別の信頼済みルート証明書リストを
持っています。Let's Encryptの新しいルート証明書「Root YE」がまだお使いの
Chromeのバージョンに反映されていない可能性が高いです(Windows自体には
既に配信済みでも、Chrome側は別スケジュールで更新されるため)。</p>
<p><strong>対処法:</strong></p>
<ol>
<li>Chromeを開き、アドレスバーに <code>chrome://settings/help</code> と入力</li>
<li>自動的に最新バージョンへの更新が始まります(既に最新の場合は「Google Chromeは最新版です」と表示)</li>
<li>更新後、Chromeを再起動してから再度アクセス</li>
</ol>
<p>サーバー側(証明書・nginx設定)に問題が無いことが確認できている場合、
Chrome側の更新で解消することが多いです。</p>
<p><strong>実際に効いた解決策:</strong> 上記を試しても直らない場合は、
<strong>Chromeをタスクバーの×だけで閉じるのではなく、一旦完全に終了して
から再度開き直す</strong>と解消することがあります(実際にこの方法で
解決した事例あり)。タスクマネージャーでChromeのプロセスが残っていないか
確認し、残っていれば終了させてから起動し直してください。</p>
<ul class="linklist">
<li>{chrome_root_store_search}</li>
</ul>

<h2>サイトが表示されない場合(DNS_PROBE_FINISHED_NXDOMAIN等)の対処方法</h2>
<p>お使いのDNS(特にCloudflareの1.1.1.1)が、ドメインの権威サーバーに
一時的に到達できないことがあります(<code>SERVFAIL — EDE: 22 (No
Reachable Authority)</code>)。この場合、Google(8.8.8.8)・Quad9(9.9.9.9)
など別のDNSでは問題なく解決できることが多いです。</p>
<p><strong>対処法:</strong></p>
<ol>
<li>スマホのモバイル回線(Wi-Fiオフ)で同じURLにアクセスしてみる(別の
DNSを経由するため、これだけで見えることが多い)</li>
<li>Windowsの場合: 設定 → ネットワークとインターネット → プロパティ
(使用中の接続) → DNSサーバーの割り当てを「手動」にし、以下を設定して保存
<ul>
<li><strong>優先DNS</strong>欄: <code>8.8.8.8</code> のみ入力(<code>8.8.4.4</code>と
まとめて<code>/</code>区切りで入力すると「無効なエントリ」エラーになるので注意)</li>
<li><strong>代替DNS</strong>欄: <code>8.8.4.4</code> を別欄に入力</li>
<li>「HTTPS経由のDNS」が「オン(手動テンプレート)」になっている場合は、
まず「オフ」にしてからシンプルな設定で保存を試す</li>
</ul>
</li>
<li>スマホでWi-Fi経由の場合(静的IP化不要): 「プライベートDNS」設定を使うと
DNSだけ変更できます。
<ul>
<li>設定 → ネットワークとインターネット(機種によっては「接続」、または
設定 → Wi-Fi → 詳細設定の中にある場合も)</li>
<li>「プライベートDNS」を探し、「プロバイダのホスト名」を選択</li>
<li><code>dns.google</code> と入力して保存</li>
</ul>
</li>
<li>それでも解決しない場合は、単純にDNSの反映待ち(通常数分〜1時間程度)
であることも多いので、時間を置いて再度アクセス</li>
</ol>
"#,
        chrome_root_store_search = google_search_link("Chrome Root Store とは", "Chrome Root Store 証明書 保護されていない通信"),
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
<li>{fermentation_yt}</li>
<li>{fermentation_g}</li>
<li>{tunnel_compost_yt}</li>
<li>{tunnel_compost_g}</li>
<li>{mercury_filter_g}</li>
<li>{mercury_filter_yt}</li>
<li>{mixed_fuel_yt}</li>
<li>{mixed_fuel_g}</li>
<li>{plastic_oil_yt}</li>
<li>{plastic_oil_g}</li>
<li>{orange_oil_yt}</li>
<li>{orange_oil_g}</li>
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
        fermentation_yt = youtube_search_link("生ごみを発酵させて肥料・燃料にする方法", "生ごみ 発酵 肥料 燃料 納豆菌"),
        fermentation_g = google_search_link("生ごみを発酵させて肥料・燃料にする方法", "生ごみ 発酵 肥料 燃料 納豆菌"),
        tunnel_compost_yt = youtube_search_link("生ごみ+燃えるごみを一緒に発酵させるトンネルコンポスト方式", "トンネルコンポスト 生ごみ 燃えるごみ 発酵 肥料 燃料"),
        tunnel_compost_g = google_search_link("生ごみ+燃えるごみを一緒に発酵させるトンネルコンポスト方式", "トンネルコンポスト 生ごみ 燃えるごみ 発酵 肥料 燃料"),
        mercury_filter_g = google_search_link("ごみ焼却時の水銀ガス回収 日立造船フィルター", "ごみ焼却 水銀ガス 回収 日立造船 フィルター"),
        mercury_filter_yt = youtube_search_link("ごみ焼却時の水銀ガス回収 日立造船フィルター", "ごみ焼却 水銀ガス 回収 日立造船 フィルター"),
        mixed_fuel_yt = youtube_search_link("燃えるごみとプラスチックごみを混ぜて燃料にする", "燃えるごみ プラスチックごみ 混ぜる 燃料"),
        mixed_fuel_g = google_search_link("燃えるごみとプラスチックごみを混ぜて燃料にする", "燃えるごみ プラスチックごみ 混ぜる 燃料"),
        plastic_oil_yt = youtube_search_link("廃プラスチックを業務用マイクロ波で石油にする", "廃プラスチック 業務用 マイクロ波 石油"),
        plastic_oil_g = google_search_link("廃プラスチックを業務用マイクロ波で石油にする", "廃プラスチック 業務用 マイクロ波 石油"),
        orange_oil_yt = youtube_search_link("発泡スチロールをオレンジオイルで溶かす", "発泡スチロール オレンジオイル 溶かす"),
        orange_oil_g = google_search_link("発泡スチロールをオレンジオイルで溶かす", "発泡スチロール オレンジオイル 溶かす"),
        aquaculture = youtube_search_link("陸上養殖(プール・水槽)魚介類 普及", "陸上養殖 プール 水槽 魚介類"),
        aquaculture_success_g = google_search_link("陸上養殖 成功例", "陸上養殖 成功例"),
        aquaculture_success_yt = youtube_search_link("陸上養殖 成功例", "陸上養殖 成功例"),
        aquaculture_failure_g = google_search_link("陸上養殖 失敗例", "陸上養殖 失敗例"),
        aquaculture_failure_yt = youtube_search_link("陸上養殖 失敗例", "陸上養殖 失敗例"),
    );
    Html(page_shell("地域・企業誘致提案 | aon.tokyo", &body))
}

/// 民間のガン治療法に関する報道記事の紹介ページ(2026-07-24追加)。
/// ユーザーから提供された実際の報道見出し・リンクをそのまま紹介するのみに留め、
/// 独自の医療的な効能・安全性の主張や推奨は一切追加しない
/// (「〜という報道があります」という紹介・引用の体裁に徹する)。
#[handler]
fn cancer_page() -> Html<String> {
    let body = format!(
        r##"<h1>民間のガン治療法に関する報道 / News on Cancer Treatment Research</h1>
<p>以下は報道・公開情報として紹介するのみで、当サイト独自の医療的な効能や安全性の主張・推奨は行っていません。
詳細は各リンク先をご覧ください。</p>
<p style="color:#555;">The items below are simply introduced as reported/published information;
this site does not add its own medical claims about efficacy or safety. Please see each link for details.</p>

<ul class="linklist">
<li><a href="https://www.facebook.com/masahiro.ishizuka.54?locale=ja_JP" target="_blank" rel="noopener noreferrer">
衝撃波で腫瘍を破壊「メスも針も使わない」肝臓がんの新治療法　大阪公立大の研究チームが特定臨床研究を開始　来年中の薬事承認を目指す</a><br>
<span style="color:#555;">Destroying Tumors with Shockwaves — a New "No Scalpel, No Needle" Liver Cancer Treatment: Osaka Metropolitan University Research Team Begins Specified Clinical Research, Aiming for Drug/Medical Device Approval Within the Next Year</span></li>

<li>マックトリガー。世界初！からだ自身が"がん治療"　九州大学が開発<br>
<span style="color:#555;">Mac Trigger. A World First! The Body Itself Fights Cancer — Developed by Kyushu University</span><br>
<a href="https://www.youtube.com/watch?v=84EkcJmgmnQ" target="_blank" rel="noopener noreferrer">YouTube</a> /
<a href="https://www.facebook.com/reel/1793445321653771?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></li>
<li>{cancer_search_jp} / {cancer_search_en}<br>
<span style="color:#555;">クリックした瞬間に検索結果を表示します(効果を保証するものではありません) / Opens a fresh search each time you click (no claims of efficacy are made here).</span></li>
<li>{cancer_video_search_jp} / {cancer_video_search_en}<br>
<span style="color:#555;">YouTube以外の動画も含めた横断検索です(効果を保証するものではありません) / A broader video search beyond YouTube (no claims of efficacy are made here).</span></li>
<li>{banana_search_jp} / {banana_search_en}<br>
<span style="color:#555;">クリックした瞬間に検索結果を表示します(効果を保証するものではありません) / Opens a fresh search each time you click (no claims of efficacy are made here).</span></li>
<li>{baking_soda_search_jp} / {baking_soda_search_en}<br>
<span style="color:#555;">クリックした瞬間に検索結果を表示します(効果を保証するものではありません) / Opens a fresh search each time you click (no claims of efficacy are made here).</span></li>
</ul>
"##,
        cancer_search_jp = youtube_search_link("がんの治療法について調べる", "がん 治療法"),
        cancer_search_en = youtube_search_link("Cancer treatment methods", "cancer treatment methods"),
        cancer_video_search_jp = google_video_search_link("がんの治療法の動画を調べる", "がん 治療法"),
        cancer_video_search_en = google_video_search_link("Cancer treatment method videos", "cancer treatment methods"),
        banana_search_jp = youtube_search_link("「バナナ ガン治療法」で調べる", "バナナ ガン治療法"),
        banana_search_en = youtube_search_link("Search \"banana cancer treatment\"", "banana cancer treatment"),
        baking_soda_search_jp = youtube_search_link("「重曹水 ガン治療法」で調べる", "重曹水 ガン治療法"),
        baking_soda_search_en = youtube_search_link("Search \"baking soda water cancer treatment\"", "baking soda water cancer treatment"),
    );
    Html(page_shell("がん治療研究に関する報道 | aon.tokyo", &body))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();
    let app = Route::new()
        .at("/", get(top))
        .at("/healthz", get(healthz))
        .at("/links", get(links_page))
        .at("/municipal", get(municipal_page))
        .at("/cancer", get(cancer_page));

    tracing::info!("aon-tokyo-server listening on 127.0.0.1:4200");
    Server::new(TcpListener::bind("127.0.0.1:4200")).run(app).await
}
