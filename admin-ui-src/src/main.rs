//! Freebuff 管理台 — "管控单"风格: 墨蓝规则线分节, 发丝线分行, 出口国家为最响元素。
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(ConsoleLayout)]
    #[route("/")]
    Overview {},
    #[route("/nodes")]
    Nodes {},
    #[route("/accounts")]
    Accounts {},
    #[route("/keys")]
    Keys {},
    #[route("/docs")]
    Docs {},
    #[route("/usage")]
    Usage {},
    #[route("/logs")]
    Logs {},
    #[route("/playground")]
    Playground {},
}

// ---------------------------------------------------------------------------
// API
// ---------------------------------------------------------------------------

async fn api_get(path: &str) -> Result<Value, String> {
    let resp = gloo_net::http::Request::get(path).send().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&resp.text().await.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

async fn api_send(method: &'static str, path: &str, body: Option<Value>) -> Result<Value, String> {
    let m = gloo_net::http::Method::from_bytes(method.as_bytes()).map_err(|e| e.to_string())?;
    let req = gloo_net::http::RequestBuilder::new(path)
        .method(m)
        .header("content-type", "application/json");
    let req = match &body {
        Some(b) => req.body(b.to_string()).map_err(|e| e.to_string())?,
        None => req.build().map_err(|e| e.to_string())?,
    };
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn body_from(pairs: &[(&str, String)]) -> Option<Value> {
    let m: serde_json::Map<String, Value> = pairs
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.to_string(), Value::String(v.clone())))
        .collect();
    if m.is_empty() { None } else { Some(Value::Object(m)) }
}

// ---------------------------------------------------------------------------
// 壳层: 常显顶部导航 (任何宽度都有按钮) + 管控单内容区
// ---------------------------------------------------------------------------

#[component]
fn ConsoleLayout() -> Element {
    rsx! {
        div { class: "min-h-screen bg-paper text-ink",
            header { class: "sticky top-0 z-30 border-b-2 border-ink bg-paper",
                div { class: "mx-auto flex w-full max-w-[1120px] flex-wrap items-center gap-x-6 gap-y-2 px-6 py-3 sm:px-10",
                    div { class: "flex items-baseline gap-2",
                        span { class: "text-lg font-semibold tracking-tight", "Freebuff" }
                        span { class: "text-xs text-ink/55", "隧道网关" }
                    }
                    nav { class: "flex flex-wrap items-center gap-1",
                        NavTab { to: Route::Overview {}, label: "总览" }
                        NavTab { to: Route::Nodes {}, label: "节点" }
                        NavTab { to: Route::Accounts {}, label: "账号" }
                        NavTab { to: Route::Keys {}, label: "API Key" }
                        NavTab { to: Route::Docs {}, label: "接入" }
                        NavTab { to: Route::Usage {}, label: "用量" }
                        NavTab { to: Route::Playground {}, label: "测试" }
                        NavTab { to: Route::Logs {}, label: "日志" }
                    }
                    div { class: "ml-auto flex items-center gap-2",
                        span { class: "h-2 w-2 rounded-full bg-alive" }
                        span { class: "text-xs text-ink/55", "网关 :8787" }
                    }
                }
            }
            main { class: "mx-auto w-full max-w-[1120px] px-6 py-8 sm:px-10",
                Outlet::<Route> {}
            }
        }
    }
}

/// 顶部导航按钮: 大点击区, 激活态墨底反白, 任何宽度可见。
#[component]
fn NavTab(to: Route, label: &'static str) -> Element {
    let active = dioxus::prelude::use_route::<Route>() == to;
    let cls = if active {
        "flex h-9 items-center rounded-sm bg-ink px-4 text-sm font-medium text-paper"
    } else {
        "flex h-9 items-center rounded-sm px-4 text-sm text-ink/60 transition-colors hover:bg-ink/[0.07] hover:text-ink"
    };
    rsx! {
        Link { to, class: cls, {label} }
    }
}

// ---------------------------------------------------------------------------
// 通用部件: 规则线分节 / 发丝线行 / 状态脊
// ---------------------------------------------------------------------------

/// 节标题: 2px 墨色上规则线, 编码"新的一节"而非装饰。
#[component]
fn Section(title: &'static str, count: Option<usize>, children: Element) -> Element {
    rsx! {
        section { class: "mt-10 first:mt-0",
            header { class: "flex items-baseline justify-between border-t-2 border-ink pt-3",
                h2 { class: "text-sm font-semibold tracking-tight", {title} }
                if let Some(c) = count {
                    span { class: "text-xs text-ink/45", {format!("{c} 项")} }
                }
            }
            div { class: "mt-4", {children} }
        }
    }
}

#[component]
fn PageHeader(title: String, desc: String) -> Element {
    rsx! {
        div { class: "mb-2",
            h1 { class: "text-2xl font-semibold tracking-tight", {title} }
            p { class: "mt-1 text-sm text-ink/55", {desc} }
        }
    }
}

/// 行: 发丝线分隔, 内容承载全部信息。
#[component]
fn Row(left: Element, right: Element) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-4 border-b border-line py-3 transition-colors first:pt-0 last:border-b-0 hover:bg-white/60",
            div { class: "min-w-0", {left} }
            div { class: "flex shrink-0 items-center gap-1.5", {right} }
        }
    }
}

/// 存活点: 语义色, 深绿/深红/灰。
#[component]
fn AliveDot(alive: Option<bool>) -> Element {
    let cls = match alive {
        Some(true) => "bg-alive",
        Some(false) => "bg-down",
        None => "bg-ink/25",
    };
    rsx! { span { class: "inline-block h-2 w-2 shrink-0 rounded-full {cls}" } }
}

#[component]
fn AliveChip(alive: Option<bool>) -> Element {
    match alive {
        Some(true) => rsx! { span { class: "rounded-sm bg-alive/12 px-1.5 py-0.5 text-xs font-medium text-alive", "存活" } },
        Some(false) => rsx! { span { class: "rounded-sm bg-down/12 px-1.5 py-0.5 text-xs font-medium text-down", "失效" } },
        None => rsx! { span { class: "rounded-sm bg-ink/[0.07] px-1.5 py-0.5 text-xs text-ink/55", "未知" } },
    }
}

#[component]
fn EmptyRow(text: &'static str) -> Element {
    rsx! {
        div { class: "border-b border-line py-8 text-center text-sm text-ink/45 last:border-b-0", {text} }
    }
}

#[component]
fn FieldInput(label: String, placeholder: &'static str, value: Signal<String>) -> Element {
    rsx! {
        label { class: "flex flex-col gap-1.5 text-xs text-ink/55",
            {label}
            Input { placeholder, class: "h-9 w-56 rounded-sm border border-line bg-white px-3 text-sm text-ink focus:border-ink focus:outline-none".to_string(), value: "{value}", on_input: move |e: dioxus::prelude::FormEvent| value.set(e.value()) }
        }
    }
}

/// 数据文本 (host/token/key): 真实机器数据用等宽, 有语义正当性。
#[component]
fn Data(text: String, class: String) -> Element {
    rsx! { span { class: "font-data {class}", {text} } }
}

fn dot_class(alive: Option<bool>) -> &'static str {
    match alive {
        Some(true) => "bg-alive",
        Some(false) => "bg-down",
        None => "bg-ink/25",
    }
}

// ---------------------------------------------------------------------------
// 总览
// ---------------------------------------------------------------------------

#[component]
fn Overview() -> Element {
    let mut data: Signal<Option<Value>> = use_signal(|| None);
    let mut nodes: Signal<Vec<Value>> = use_signal(Vec::new);
    let mut error: Signal<String> = use_signal(String::new);
    use_future(move || async move {
        if let Ok(v) = api_get("/admin/overview").await {
            data.set(Some(v));
        } else {
            error.set("网关无响应".into());
        }
        if let Ok(v) = api_get("/admin/nodes").await {
            nodes.set(v["relay"].as_array().cloned().unwrap_or_default());
        }
    });

    let total_nodes = nodes().len();
    let alive_nodes = nodes().iter().filter(|n| n["alive"].as_bool().unwrap_or(false)).count();

    rsx! {
        PageHeader { title: "总览".to_string(), desc: "节点出口、账号池与模型一览".to_string() }
        if !error().is_empty() {
            div { class: "mt-4 border-l-[3px] border-down bg-down/[0.06] px-4 py-3 text-sm text-down", {error()} }
        }
        match data() {
            Some(v) => rsx! {
                Section { title: "出口状态", count: Some(total_nodes),
                    // 单条摘要: 网关实际出口 + 存活统计 (节点明细在「节点」页)
                    div { class: "flex flex-wrap items-center gap-x-4 gap-y-1 border-l-[3px] border-ink bg-white px-4 py-3 text-sm",
                        match v["tunnel"].as_object() {
                            Some(t) => rsx! {
                                span { class: "font-semibold text-ink",
                                    {format!("{} · {}", t["scheme"].as_str().unwrap_or(""), t["host"].as_str().unwrap_or(""))} }
                                span { class: "text-alive", "直连出站" }
                            },
                            None => rsx! {
                                span { class: "font-semibold text-ink/40", "未启用隧道" }
                                span { class: "text-ink/55", "本机直连出口" }
                            },
                        }
                        span { class: "ml-auto text-xs text-ink/55", {format!("节点 {total_nodes} · 存活 {alive_nodes}")} }
                    }
                }
                div { class: "grid gap-x-10 lg:grid-cols-2",
                    Section { title: "账号池", count: Some(v["accounts"]["accounts"].as_u64().unwrap_or(0) as usize),
                        for d in v["accounts"]["account_details"].as_array().cloned().unwrap_or_default() {
                            Row {
                                left: rsx! {
                                    div { class: "flex items-center gap-2.5",
                                        AliveDot { alive: d["alive"].as_bool() }
                                        Data { text: d["token"].as_str().unwrap_or("").to_string(), class: "text-sm".to_string() }
                                    }
                                },
                                right: rsx! { AliveChip { alive: d["alive"].as_bool() } }
                            }
                        }
                        if v["accounts"]["account_details"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
                            EmptyRow { text: "暂无账号" }
                        }
                    }
                    Section { title: "模型注册表", count: Some(v["models"].as_array().map(|a| a.len()).unwrap_or(0)),
                        div { class: "flex flex-wrap gap-x-4 gap-y-1.5 pb-1",
                            for m in v["models"].as_array().cloned().unwrap_or_default() {
                                Data { text: m.as_str().unwrap_or("").to_string(), class: "text-xs text-ink/70".to_string() }
                            }
                        }
                    }
                }
                Section { title: "签发凭据", count: Some(v["keys"].as_array().map(|a| a.len()).unwrap_or(0)),
                    p { class: "pb-1 text-sm text-ink/55", "API Key 用于网关鉴权, 节点隧道承载流量。" }
                }
            },
            None => rsx! { div { class: "mt-6 text-sm text-ink/45", "读取中…" } },
        }
    }
}

// ---------------------------------------------------------------------------
// 节点
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum NodeFilter {
    All,
    Alive,
    Dead,
}

#[component]
fn Nodes() -> Element {
    let mut nodes: Signal<Vec<Value>> = use_signal(Vec::new);
    let error: Signal<String> = use_signal(String::new);
    let mut probing: Signal<Option<u64>> = use_signal(|| None);
    let mut probes: Signal<HashMap<u64, Value>> = use_signal(HashMap::new);
    let mut filter: Signal<NodeFilter> = use_signal(|| NodeFilter::All);

    let load = move |mut nodes: Signal<Vec<Value>>| {
        spawn(async move {
            if let Ok(v) = api_get("/admin/nodes").await {
                nodes.set(v["relay"].as_array().cloned().unwrap_or_default());
            }
        });
    };
    use_future({ let nodes = nodes.clone(); move || { load(nodes); async {} } });

    let total = nodes().len();
    let alive_n = nodes().iter().filter(|n| n["alive"].as_bool().unwrap_or(false)).count();
    let dead_n = total - alive_n;

    let shown: Vec<Value> = nodes().iter().filter(|n| match filter() {
        NodeFilter::All => true,
        NodeFilter::Alive => n["alive"].as_bool().unwrap_or(false),
        NodeFilter::Dead => !n["alive"].as_bool().unwrap_or(false),
    }).cloned().collect();

    rsx! {
        PageHeader { title: "节点".to_string(), desc: "粘贴分享链接, 本地自动拉起隧道; 测活查看真实出口".to_string() }
        ImportCard { nodes: nodes.clone(), error }

        Section { title: "已导入节点", count: Some(total),
            // 筛选页签 + 批量测活
            div { class: "mb-4 flex items-center justify-between",
                div { class: "flex gap-1",
                    TabBtn { label: "全部", count: total, active: filter() == NodeFilter::All,
                        on_click: move |_| filter.set(NodeFilter::All) }
                    TabBtn { label: "存活", count: alive_n, active: filter() == NodeFilter::Alive,
                        on_click: move |_| filter.set(NodeFilter::Alive) }
                    TabBtn { label: "失效", count: dead_n, active: filter() == NodeFilter::Dead,
                        on_click: move |_| filter.set(NodeFilter::Dead) }
                }
                Button { variant: ButtonVariant::Outline, class: "h-8 rounded-sm text-xs",
                    disabled: alive_n == 0,
                    on_click: move |_| {
                        let all = nodes().clone();
                        let probes_c = probes.clone();
                        let probing_c = probing.clone();
                        for n in all {
                            if n["alive"].as_bool().unwrap_or(false) {
                                let port = n["local_port"].as_u64().unwrap_or(0);
                                let mut probes = probes_c.clone();
                                let mut probing = probing_c.clone();
                                probing.set(Some(port));
                                spawn(async move {
                                    if let Ok(v) = api_send("POST", &format!("/admin/nodes/{port}/probe"), None).await {
                                        probes.write().insert(port, v);
                                    }
                                    probing.set(None);
                                });
                            }
                        }
                    },
                    "批量测活" }
            }
            // 卡片网格: 一屏三列
            div { class: "grid grid-cols-[repeat(auto-fill,minmax(280px,340px))] gap-3",
                if shown.is_empty() {
                    div { class: "col-span-full border-b border-line py-8 text-center text-sm text-ink/45",
                        match filter() {
                            NodeFilter::All => "暂无节点 — 当前直连出口",
                            NodeFilter::Alive => "没有存活的节点",
                            NodeFilter::Dead => "没有失效的节点",
                        }
                    }
                }
                for n in shown {
                    NodeCard { n: n.clone(), probes, probing, on_changed: move |_| load(nodes) }
                }
            }
        }
    }
}

/// 筛选页签。
#[component]
fn TabBtn(label: &'static str, count: usize, active: bool, on_click: EventHandler<dioxus::prelude::MouseEvent>) -> Element {
    let cls = if active {
        "h-8 rounded-sm bg-ink px-3 text-xs font-medium text-paper"
    } else {
        "h-8 rounded-sm px-3 text-xs text-ink/60 hover:bg-ink/[0.06] hover:text-ink"
    };
    rsx! {
        button { class: cls, onclick: move |e| on_click.call(e), {format!("{label} {count}")} }
    }
}

/// 节点卡: 网格单元, 探测结果内嵌。
#[component]
fn NodeCard(n: Value, mut probes: Signal<HashMap<u64, Value>>, mut probing: Signal<Option<u64>>, on_changed: EventHandler<Value>) -> Element {
    let full_link = n["full_link"].as_str().unwrap_or("").to_string();
    let scheme = n["scheme"].as_str().unwrap_or("").to_string();
    let alive = n["alive"].as_bool().unwrap_or(false);
    let port = n["local_port"].as_u64().unwrap_or(0);
    let is_probing = probing() == Some(port);
    let result = probes.read().get(&port).cloned();
    rsx! {
        div { class: "flex flex-col border border-line bg-white transition-colors hover:border-ink/45",
            // 头: 协议标签脊 + 存活
            div { class: "flex items-center justify-between border-b border-line border-l-[3px] border-l-ink px-3 py-2",
                span { class: "text-sm font-semibold tracking-tight text-ink", {scheme} }
                AliveDot { alive: Some(alive) }
            }
            // 体: 链接与出站
            div { class: "px-3 py-2.5",
                Data { text: trunc_link(&full_link, 44), class: "block truncate text-xs text-ink/60".to_string() }
                if alive {
                    Data { text: format!("出站 127.0.0.1:{port}"), class: "mt-1 block text-xs text-ink/45".to_string() }
                }
            }
            // 探测结果 (内嵌)
            if let Some(v) = result {
                div { class: "border-t border-line bg-paper/60 px-3 py-2 text-xs",
                    ProbeInline { result: v }
                }
            }
            // 底: 操作
            div { class: "mt-auto flex items-center gap-2 border-t border-line px-3 py-2",
                Button { variant: ButtonVariant::Outline, class: "h-7 rounded-sm text-xs", disabled: is_probing,
                    on_click: move |_| {
                        probing.set(Some(port));
                        spawn(async move {
                            if let Ok(v) = api_send("POST", &format!("/admin/nodes/{port}/probe"), None).await {
                                probes.write().insert(port, v);
                            }
                            probing.set(None);
                        });
                    },
                    if is_probing { "探测中" } else { "测活" } }
                if alive {
                    Button { variant: ButtonVariant::Ghost, class: "h-7 rounded-sm text-xs",
                        on_click: move |_| {
                            spawn(async move {
                                let _ = api_send("POST", &format!("/admin/nodes/{port}/stop"), None).await;
                                on_changed.call(Value::Null);
                            });
                        },
                        "停止" }
                }
            }
        }
    }
}

/// 卡片内嵌探测结果 (紧凑行)。
#[component]
fn ProbeInline(result: Value) -> Element {
    let alive = result["alive"].as_bool();
    let lat = result["latency_ms"].as_u64();
    let s = &result["session"];
    let has_session = s.is_object();
    let cc = s["countryCode"].as_str().unwrap_or("").to_string();
    let model = s["model"].as_str().unwrap_or("").to_string();
    let restricted = result["restricted"].as_bool().unwrap_or(false);
    let err_text = result["error"].as_str().map(String::from);
    rsx! {
        div { class: "flex flex-wrap items-center gap-x-3 gap-y-0.5",
            span { class: "text-[10px] font-medium text-ink/40", "测活" }
            match alive {
                Some(true) => rsx! { span { class: "font-medium text-alive", "连通" } },
                Some(false) => rsx! { span { class: "font-medium text-down", "不通" } },
                None => rsx! { span { class: "text-ink/40", "未探测" } },
            }
            if let Some(l) = lat {
                span { class: "text-ink/55", {format!("{l}ms")} }
            }
            if has_session {
                if restricted {
                    span { class: "text-amber-700", {format!("受限 仅{model}")} }
                } else {
                    span { class: "text-alive", "完整访问" }
                }
                if !cc.is_empty() {
                    span { class: "font-semibold text-ink", {format!("出口 {cc}")} }
                }
            }
            if let Some(e) = err_text {
                span { class: "text-down", {e} }
            }
        }
    }
}

fn trunc_link(s: &str, n: usize) -> String {
    if let Some((scheme, rest)) = s.split_once("://") {
        let host = rest.rsplit('@').next().unwrap_or(rest);
        let shown = format!("{}://{}", scheme, host);
        if shown.len() > n { format!("{}...", &shown[..n]) } else { shown }
    } else if s.len() > n {
        format!("{}...", &s[..n])
    } else {
        s.to_string()
    }
}

#[component]
fn ProbeBody(title: String, result: Value) -> Element {
    let alive = result["alive"].as_bool();
    let lat = result["latency_ms"].as_u64();
    let s = &result["session"];
    let has_session = s.is_object();
    let tier = s["accessTier"].as_str().unwrap_or("").to_string();
    let cc = s["countryCode"].as_str().unwrap_or("").to_string();
    let model = s["model"].as_str().unwrap_or("").to_string();
    let restricted = result["restricted"].as_bool().unwrap_or(false);
    let err_text = result["error"].as_str().map(String::from);
    rsx! {
        div { class: "border-l-[3px] border-ink bg-white px-4 py-3",
            div { class: "flex flex-wrap items-center gap-x-5 gap-y-1 text-sm",
                span { class: "font-medium text-ink", {title} }
                match alive {
                    Some(true) => rsx! { span { class: "text-alive", "连通" } },
                    Some(false) => rsx! { span { class: "text-down", "不通" } },
                    None => rsx! { span { class: "text-ink/40", "未探测" } },
                }
                if let Some(l) = lat {
                    span { class: "text-ink/55", {format!("{l}ms")} }
                }
                if has_session {
                    if restricted {
                        span { class: "text-amber-700", {format!("受限: 仅 {model}")} }
                    } else {
                        span { class: "text-alive", "完整访问" }
                    }
                    if !cc.is_empty() {
                        span { class: "font-medium text-ink", {format!("出口 {cc}")} }
                    }
                    if !tier.is_empty() {
                        span { class: "text-ink/55", {format!("层级 {tier}")} }
                    }
                }
                if let Some(e) = err_text {
                    span { class: "text-down", {e} }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 快速导入
// ---------------------------------------------------------------------------

#[component]
fn ImportCard(mut nodes: Signal<Vec<Value>>, mut error: Signal<String>) -> Element {
    let mut import_text = use_signal(String::new);
    let mut importing = use_signal(|| false);
    let mut result: Signal<Option<Value>> = use_signal(|| None);
    rsx! {
        div { class: "border border-line bg-white p-5",
            div { class: "flex items-baseline justify-between",
                div { class: "text-sm font-semibold text-ink", "快速导入" }
                span { class: "text-xs text-ink/50", "一行一个, 支持 socks5 / hy2 / trojan / ss / vless / vmess / tuic" }
            }
            textarea {
                class: "mt-3 h-28 w-full resize-y rounded-sm border border-line bg-paper px-3 py-2 font-data text-xs text-ink placeholder:text-ink/35 focus:border-ink focus:outline-none",
                placeholder: "socks5://user:pass@1.2.3.4:1080\nhy2://pass@host:443?insecure=1\ntrojan://pass@host:443",
                value: "{import_text}",
                oninput: move |e: dioxus::prelude::FormEvent| import_text.set(e.value()),
            }
            div { class: "mt-3 flex items-center gap-3",
                Button { variant: ButtonVariant::Primary, class: "h-8 rounded-sm", disabled: importing(),
                    icon_left: rsx! { Upload { class: "size-3.5" } },
                    on_click: move |_| {
                        let body = body_from(&[("text", import_text())]);
                        importing.set(true);
                        spawn(async move {
                            match api_send("POST", "/admin/nodes/import", body).await {
                                Ok(v) => {
                                    result.set(Some(v.clone()));
                                    if let Ok(list) = api_get("/admin/nodes").await {
                                        nodes.set(list["relay"].as_array().cloned().unwrap_or_default());
                                    }
                                    import_text.set(String::new());
                                }
                                Err(e) => error.set(e),
                            }
                            importing.set(false);
                        });
                    },
                    if importing() { "导入中" } else { "批量导入" } }
                if !error().is_empty() {
                    span { class: "text-xs text-down", {error()} }
                }
            }
            if let Some(v) = result() {
                div { class: "mt-3 border-t border-line pt-3 text-xs",
                    div { class: "flex flex-wrap gap-x-5 gap-y-1",
                        span { class: "font-medium text-alive", {format!("成功 {}", v["imported_count"].as_u64().unwrap_or(0))} }
                        span { class: "text-ink/55", {format!("重复跳过 {}", v["skipped"].as_u64().unwrap_or(0))} }
                        if let Some(errs) = v["errors"].as_array() {
                            if !errs.is_empty() {
                                span { class: "font-medium text-down", {format!("失败 {}", errs.len())} }
                            }
                        }
                    }
                    if let Some(errs) = v["errors"].as_array() {
                        for e in errs {
                            Data { text: e.as_str().unwrap_or("").to_string(), class: "mt-1 block truncate text-down".to_string() }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 账号
// ---------------------------------------------------------------------------

#[component]
fn Accounts() -> Element {
    let mut accounts: Signal<Value> = use_signal(|| Value::Null);
    let mut probe: Signal<Option<Value>> = use_signal(|| None);
    let mut probing: Signal<Option<String>> = use_signal(|| None);
    let mut dialog_open = use_signal(|| false);

    let load = move |mut accounts: Signal<Value>| {
        spawn(async move {
            if let Ok(v) = api_get("/admin/accounts").await {
                accounts.set(v);
            }
        });
    };
    use_future({ let accounts = accounts.clone(); move || { load(accounts); async {} } });

    rsx! {
        PageHeader { title: "账号".to_string(), desc: "codebuff 账号 token 池 — 会话钉住与轮换".to_string() }
        div { class: "border border-line bg-white p-5",
            div { class: "flex flex-wrap items-center justify-between gap-3",
                div {
                    div { class: "text-sm font-semibold text-ink", "添加账号" }
                    p { class: "mt-0.5 text-xs text-ink/55", "在线授权自动收录, 或手动粘贴 token" }
                }
                Button { variant: ButtonVariant::Primary, class: "h-9 rounded-sm",
                    icon_left: rsx! { Plus { class: "size-3.5" } },
                    on_click: move |_| dialog_open.set(true),
                    "添加账号" }
            }
        }
        AuthDialog { open: dialog_open, on_loaded: move |_| load(accounts) }
        {
            let all = accounts()["account_details"].as_array().cloned().unwrap_or_default();
            let freebuff: Vec<Value> = all.iter().filter(|d| d["source"].as_str() != Some("codebuff")).cloned().collect();
            let codebuff: Vec<Value> = all.iter().filter(|d| d["source"].as_str() == Some("codebuff")).cloned().collect();
            rsx! {
                Section { title: "Freebuff 账号池", count: Some(freebuff.len()),
                    if freebuff.is_empty() {
                        EmptyRow { text: "暂无 — 「添加账号 → Freebuff 账号」在线授权或粘贴 token" }
                    }
                    for d in freebuff {
                        AccountRow { d: d.clone(), probe, probing, on_changed: move |_| load(accounts) }
                    }
                }
                Section { title: "Codebuff 账号池", count: Some(codebuff.len()),
                    if codebuff.is_empty() {
                        EmptyRow { text: "暂无 — 「添加账号 → Codebuff 账号」" }
                    }
                    for d in codebuff {
                        AccountRow { d: d.clone(), probe, probing, on_changed: move |_| load(accounts) }
                    }
                }
            }
        }
        if let Some(v) = probe() {
            Section { title: "受限查询", count: None,
                ProbeBody { title: "该账号当前会话".to_string(), result: v }
            }
        }
    }
}

/// 添加账号弹窗: 两种方式 — 在线授权 / 粘贴 token。
#[component]
fn AuthDialog(mut open: Signal<bool>, on_loaded: EventHandler<Value>) -> Element {
    let mut mode = use_signal(|| 0usize); // 0=在线授权 1=粘贴token
    let mut provider = use_signal(|| "freebuff".to_string());
    let mut token = use_signal(String::new);
    let mut auth_state = use_signal(|| String::new());
    let mut auth_error = use_signal(String::new);
    let mut authorizing = use_signal(|| false);

    let mut flow_id = use_signal(String::new);

    let start_auth = move |_| {
        spawn(async move {
            auth_error.set(String::new());
            auth_state.set("正在发起授权…".into());
            match api_send("POST", "/admin/auth/start", Some(serde_json::json!({"provider": provider()}))).await {
                Ok(v) => {
                    let url = v["url"].as_str().unwrap_or("").to_string();
                    let fid = v["flowId"].as_str().unwrap_or("").to_string();
                    flow_id.set(fid);
                    if !url.is_empty() {
                        open_new_tab(&url);
                    }
                    auth_state.set("已打开授权页 — 完成登录后本页自动收录".into());
                    // 轮询
                    for _ in 0..240 {
                        js_sleep(2500).await;
                        if !authorizing() {
                            break;
                        }
                        match api_get(&format!("/admin/auth/{}", flow_id())).await {
                            Ok(s) => {
                                let status = s["status"].as_str().unwrap_or("");
                                if status == "complete" {
                                    auth_state.set("授权成功 — 账号已收录".into());
                                    authorizing.set(false);
                                    on_loaded.call(Value::Null);
                                    break;
                                } else if status == "expired" {
                                    auth_state.set("授权码已过期 — 请重新发起".into());
                                    authorizing.set(false);
                                    break;
                                }
                            }
                            Err(e) => {
                                auth_error.set(e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => auth_error.set(e),
            }
        });
    };

    if !open() {
        return rsx! {};
    }
    rsx! {
        div { class: "fixed inset-0 z-40 flex items-center justify-center bg-ink/45 p-4",
            div { class: "w-full max-w-md border border-line bg-white",
                div { class: "flex items-center justify-between border-b border-line px-4 py-3",
                    span { class: "text-sm font-semibold text-ink", "添加账号" }
                    button {
                        class: "text-lg leading-none text-ink/50 hover:text-ink",
                        onclick: move |_| open.set(false),
                        "×" }
                }
                // 两种方式切换
                div { class: "flex gap-1 border-b border-line px-4 pt-3",
                    button {
                        class: if mode() == 0 { "rounded-t-sm border-b-2 border-ink px-3 pb-2 text-xs font-medium text-ink" } else { "px-3 pb-2 text-xs text-ink/50" },
                        onclick: move |_| mode.set(0),
                        "在线授权" }
                    button {
                        class: if mode() == 1 { "rounded-t-sm border-b-2 border-ink px-3 pb-2 text-xs font-medium text-ink" } else { "px-3 pb-2 text-xs text-ink/50" },
                        onclick: move |_| mode.set(1),
                        "粘贴 Token" }
                }
                if mode() == 0 {
                    div { class: "px-4 py-4",
                        p { class: "text-sm text-ink/70", "选择授权体系并跳转完成登录，本页自动收录账号凭证。" }
                        div { class: "mt-3 flex gap-2",
                            button {
                                class: if provider() == "freebuff" { "h-9 flex-1 rounded-sm border-2 border-ink bg-ink/[0.06] text-sm font-medium text-ink" } else { "h-9 flex-1 rounded-sm border border-line text-sm text-ink/55 hover:border-ink/35" },
                                onclick: move |_| provider.set("freebuff".to_string()),
                                "Freebuff 账号" }
                            button {
                                class: if provider() == "codebuff" { "h-9 flex-1 rounded-sm border-2 border-ink bg-ink/[0.06] text-sm font-medium text-ink" } else { "h-9 flex-1 rounded-sm border border-line text-sm text-ink/55 hover:border-ink/35" },
                                onclick: move |_| provider.set("codebuff".to_string()),
                                "Codebuff 账号" }
                        }
                        p { class: "mt-1.5 text-xs text-ink/45",
                            if provider() == "freebuff" { "将打开 freebuff.com/login" } else { "将打开 www.codebuff.com/login" } }
                        div { class: "mt-4 flex items-center gap-3",
                            Button { variant: ButtonVariant::Primary, class: "h-9 rounded-sm",
                                disabled: authorizing(),
                                on_click: {
                                    let mut authorizing = authorizing.clone();
                                    move |_| {
                                        authorizing.set(true);
                                        start_auth(());
                                    }
                                },
                                "打开授权页面" }
                            if authorizing() {
                                span { class: "inline-block h-2 w-2 animate-pulse rounded-full bg-alive" }
                            }
                        }
                        if !auth_state().is_empty() {
                            p { class: "mt-3 text-xs text-ink/55", {auth_state()} }
                        }
                        if !auth_error().is_empty() {
                            p { class: "mt-2 border-l-[3px] border-down bg-down/[0.06] px-3 py-2 text-xs text-down", {auth_error()} }
                        }
                    }
                } else {
                    div { class: "px-4 py-4",
                        p { class: "mb-2 text-xs text-ink/55",
                            if provider() == "freebuff" { "将收入 Freebuff 账号池" } else { "将收入 Codebuff 账号池" } }
                        label { class: "flex flex-col gap-1.5 text-xs text-ink/55",
                            "authToken"
                            Input { placeholder: "粘贴 codebuff authToken", class: "h-9 rounded-sm border border-line bg-paper px-3 font-data text-xs text-ink focus:border-ink focus:outline-none".to_string(), value: "{token}", on_input: move |e: dioxus::prelude::FormEvent| token.set(e.value()) }
                        }
                        div { class: "mt-4",
                            Button { variant: ButtonVariant::Primary, class: "h-9 rounded-sm",
                                on_click: move |_| {
                                    let body = body_from(&[("token", token()), ("provider", provider())]);
                                    spawn(async move {
                                        let _ = api_send("POST", "/admin/accounts", body).await;
                                        token.set(String::new());
                                        open.set(false);
                                        on_loaded.call(Value::Null);
                                    });
                                },
                                "收录账号" }
                        }
                    }
                }
            }
        }
    }
}

async fn js_sleep(ms: i32) {
    let p = js_sys::Promise::new(&mut |resolve, _| {
        window_set_timeout(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(p).await;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn setTimeout(closure: &js_sys::Function, ms: i32);
}

fn window_set_timeout(f: &js_sys::Function, ms: i32) {
    setTimeout(f, ms);
}

fn open_new_tab(url: &str) {
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = window)]
        fn open(url: &str, target: &str);
    }
    open(url, "_blank");
}

#[component]
fn AccountRow(d: Value, mut probe: Signal<Option<Value>>, mut probing: Signal<Option<String>>, on_changed: EventHandler<Value>) -> Element {
    let mut acc_confirm_open = use_signal(|| false);
    let mut rename_open = use_signal(|| false);
    let head = d["token"].as_str().unwrap_or("").to_string();
    let head_query = head.trim_end_matches("...").to_string();
    let is_probing = probing() == Some(head_query.clone());
    let head_del = head_query.clone();
    rsx! {
        Row {
            left: rsx! {
                div { class: "flex min-w-0 items-center gap-2.5",
                    AliveDot { alive: d["alive"].as_bool() }
                    Data { text: head, class: "truncate text-sm".to_string() }
                }
            },
            right: rsx! {
                AliveChip { alive: d["alive"].as_bool() }
                Button { variant: ButtonVariant::Outline, class: "h-7 rounded-sm text-xs", disabled: is_probing,
                    on_click: move |_| {
                        let hq = head_query.clone();
                        probing.set(Some(hq.clone()));
                        spawn(async move {
                            if let Ok(v) = api_send("POST", &format!("/admin/accounts/{hq}/probe"), None).await {
                                probe.set(Some(v));
                            }
                            probing.set(None);
                        });
                    },
                    if is_probing { "查询中" } else { "受限查询" } }
                Button { variant: ButtonVariant::Ghost, class: "h-7 rounded-sm text-xs text-down",
                    on_click: move |_| acc_confirm_open.set(true),
                    "删除" }
                ConfirmDeleteDialog { open: acc_confirm_open, title: "删除账号".to_string(), target: d["token"].as_str().unwrap_or("").to_string(),
                    on_confirm: move |_| {
                        let hd = head_del.clone();
                        spawn(async move {
                            let _ = api_send("DELETE", &format!("/admin/accounts/{hd}"), None).await;
                            on_changed.call(Value::Null);
                        });
                    } }
                RenameDialog { open: rename_open, title: "账号命名".to_string(), initial: d["alias"].as_str().unwrap_or("").to_string(),
                    on_save: move |name: String| {
                        let hd = head_del.clone();
                        spawn(async move {
                            let _ = api_send("PATCH", &format!("/admin/accounts/{hd}/alias"),
                                Some(serde_json::json!({"name": name}))).await;
                            on_changed.call(Value::Null);
                        });
                    } }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Key
// ---------------------------------------------------------------------------

#[component]
fn Keys() -> Element {
    let mut keys: Signal<Vec<Value>> = use_signal(Vec::new);
    let mut new_key: Signal<Option<String>> = use_signal(|| None);
    let mut copied: Signal<bool> = use_signal(|| false);
    let mut create_open = use_signal(|| false);

    let load = move |mut keys: Signal<Vec<Value>>| {
        spawn(async move {
            if let Ok(v) = api_get("/admin/keys").await {
                keys.set(v["keys"].as_array().cloned().unwrap_or_default());
            }
        });
    };
    use_future({ let keys = keys.clone(); move || { load(keys); async {} } });

    rsx! {
        PageHeader { title: "API Key".to_string(), desc: "请求头 Authorization: Bearer <key> 或 x-api-key: <key>" }
        div { class: "border border-line bg-white p-5",
            Button { variant: ButtonVariant::Primary, class: "h-8 rounded-sm",
                icon_left: rsx! { Plus { class: "size-3.5" } },
                on_click: move |_| create_open.set(true),
                "签发新 Key" }
            KeyCreateDialog { open: create_open, new_key, copied, on_created: move |_| load(keys) }
            if let Some(k) = new_key() {
                div { class: "mt-4 border-l-[3px] border-alive bg-alive/[0.06] px-4 py-3",
                    div { class: "text-xs text-ink/55", "新 Key 只完整显示这一次" }
                    div { class: "mt-1 flex items-center justify-between gap-3",
                        Data { text: k.clone(), class: "text-sm".to_string() }
                        Button { variant: ButtonVariant::Outline, class: "h-7 shrink-0 rounded-sm text-xs",
                            on_click: move |_| {
                                let text = new_key().unwrap_or_default();
                                spawn(async move {
                                    copy_to_clipboard(&text);
                                    copied.set(true);
                                });
                            },
                            if copied() { "已复制" } else { "复制" } }
                    }
                }
            }
        }
        Section { title: "已签发", count: Some(keys().len()),
            if keys().is_empty() {
                EmptyRow { text: "暂无 Key — 网关当前不鉴权" }
            }
            for k in keys() {
                KeyRow { k: k.clone(), on_changed: move |_| load(keys) }
            }
        }
    }
}

#[component]
/// 创建 Key 弹窗: 输入名字后签发 (空名后端自动编号)
#[component]
fn KeyCreateDialog(
    mut open: Signal<bool>,
    mut new_key: Signal<Option<String>>,
    mut copied: Signal<bool>,
    on_created: EventHandler<Value>,
) -> Element {
    let mut name = use_signal(String::new);
    let mut creating = use_signal(|| false);

    if !open() {
        return rsx! {};
    }
    rsx! {
        div { class: "fixed inset-0 z-40 flex items-center justify-center bg-ink/45 p-4",
            div { class: "w-full max-w-md border border-line bg-white",
                div { class: "flex items-center justify-between border-b border-line px-4 py-3",
                    span { class: "text-sm font-semibold text-ink", "签发新 API Key" }
                    button {
                        class: "text-lg leading-none text-ink/50 hover:text-ink",
                        onclick: move |_| open.set(false),
                        "×" }
                }
                div { class: "px-4 py-4",
                    label { class: "flex flex-col gap-1.5 text-xs text-ink/55",
                        "名字 (如: my-phone, 可留空自动编号)"
                        input {
                            class: "h-9 border border-line px-3 text-sm text-ink placeholder:text-ink/30 focus:border-ink focus:outline-none",
                            placeholder: "key-1",
                            value: name(),
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div { class: "mt-4 flex justify-end gap-2",
                        Button { variant: ButtonVariant::Outline, class: "h-9 rounded-sm",
                            on_click: move |_| open.set(false),
                            "取消" }
                        Button { variant: ButtonVariant::Primary, class: "h-9 rounded-sm",
                            disabled: creating(),
                            on_click: move |_| {
                                creating.set(true);
                                let body = if name().trim().is_empty() {
                                    None
                                } else {
                                    Some(serde_json::json!({"name": name().trim()}))
                                };
                                spawn(async move {
                                    if let Ok(v) = api_send("POST", "/admin/keys", body).await {
                                        new_key.set(v["key"].as_str().map(String::from));
                                        copied.set(false);
                                        open.set(false);
                                        name.set(String::new());
                                        on_created.call(Value::Null);
                                    }
                                    creating.set(false);
                                });
                            },
                            if creating() { "签发中…" } else { "签发" } }
                    }
                }
            }
        }
    }
}

/// 删除确认弹窗 (通用): 显示目标名, 确认后才执行
#[component]
fn ConfirmDeleteDialog(
    mut open: Signal<bool>,
    title: String,
    target: String,
    on_confirm: EventHandler<()>,
) -> Element {
    if !open() {
        return rsx! {};
    }
    rsx! {
        div { class: "fixed inset-0 z-40 flex items-center justify-center bg-ink/45 p-4",
            div { class: "w-full max-w-sm border border-line bg-white",
                div { class: "border-b border-line px-4 py-3",
                    span { class: "text-sm font-semibold text-ink", {title} }
                }
                div { class: "px-4 py-4",
                    p { class: "text-sm text-ink/70", "即将删除:" }
                    p { class: "mt-1 border-l-[3px] border-down bg-down/[0.06] px-3 py-1.5 text-sm text-ink", {target} }
                    p { class: "mt-2 text-xs text-ink/45", "此操作不可撤销。" }
                    div { class: "mt-4 flex justify-end gap-2",
                        Button { variant: ButtonVariant::Outline, class: "h-9 rounded-sm",
                            on_click: move |_| open.set(false),
                            "取消" }
                        Button { variant: ButtonVariant::Primary, class: "h-9 rounded-sm bg-down text-white hover:bg-down/90",
                            on_click: move |_| {
                                open.set(false);
                                on_confirm.call(());
                            },
                            "确认删除" }
                    }
                }
            }
        }
    }
}

#[component]
/// 通用改名弹窗: 输入名字 → on_save (空名 = 清除别名)
#[component]
fn RenameDialog(
    mut open: Signal<bool>,
    title: String,
    initial: String,
    on_save: EventHandler<String>,
) -> Element {
    let mut name = use_signal(String::new);
    let mut inited = use_signal(|| false);
    if !open() {
        inited.set(false);
        return rsx! {};
    }
    if !inited() {
        name.set(initial.clone());
        inited.set(true);
    }
    rsx! {
        div { class: "fixed inset-0 z-40 flex items-center justify-center bg-ink/45 p-4",
            div { class: "w-full max-w-sm border border-line bg-white",
                div { class: "border-b border-line px-4 py-3",
                    span { class: "text-sm font-semibold text-ink", {title} }
                }
                div { class: "px-4 py-4",
                    input {
                        class: "h-9 w-full border border-line px-3 text-sm text-ink placeholder:text-ink/30 focus:border-ink focus:outline-none",
                        placeholder: "输入名字 (清空 = 恢复显示 token)",
                        value: name(),
                        oninput: move |e| name.set(e.value()),
                    }
                    div { class: "mt-4 flex justify-end gap-2",
                        Button { variant: ButtonVariant::Outline, class: "h-9 rounded-sm",
                            on_click: move |_| open.set(false),
                            "取消" }
                        Button { variant: ButtonVariant::Primary, class: "h-9 rounded-sm",
                            on_click: move |_| {
                                open.set(false);
                                on_save.call(name().trim().to_string());
                            },
                            "保存" }
                    }
                }
            }
        }
    }
}

fn KeyRow(k: Value, on_changed: EventHandler<Value>) -> Element {
    let key = k["key"].as_str().unwrap_or("").to_string();
    let masked = mask_key(&key);
    let mut confirm_open = use_signal(|| false);
    let enabled = k["enabled"].as_bool().unwrap_or(true);
    rsx! {
        Row {
            left: rsx! {
                div { class: "min-w-0",
                    div { class: "flex items-center gap-2",
                        AliveDot { alive: Some(enabled) }
                        span { class: "text-sm text-ink", {k["name"].as_str().unwrap_or("").to_string()} }
                        span { class: "text-[10px] text-ink/40", {if enabled { "启用" } else { "已禁用" }} }
                    }
                    Data { text: masked, class: "text-xs text-ink/50".to_string() }
                }
            },
            right: rsx! {
                Button { variant: ButtonVariant::Ghost, class: "h-7 rounded-sm text-xs",
                    on_click: move |_| {
                        let key2 = key.clone();
                        spawn(async move {
                            let _ = api_send("PATCH", &format!("/admin/keys/{key2}/toggle"),
                                Some(serde_json::json!({"enabled": !enabled}))).await;
                            on_changed.call(Value::Null);
                        });
                    },
                    if enabled { "禁用" } else { "启用" } }
                Button { variant: ButtonVariant::Ghost, class: "h-7 rounded-sm text-xs text-down",
                    on_click: move |_| confirm_open.set(true),
                    "删除" }
                ConfirmDeleteDialog { open: confirm_open, title: "删除 API Key".to_string(), target: k["name"].as_str().unwrap_or("").to_string(),
                    on_confirm: move |_| {
                        let key = key.clone();
                        spawn(async move {
                            let _ = api_send("DELETE", &format!("/admin/keys/{key}"), None).await;
                        });
                        on_changed.call(Value::Null);
                    } }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 接入文档
// ---------------------------------------------------------------------------

#[component]
fn Docs() -> Element {
    let mut copied: Signal<Option<&'static str>> = use_signal(|| None);
    let base = "http://127.0.0.1:8787";
    rsx! {
        PageHeader { title: "接入".to_string(), desc: "OpenAI 与 Anthropic 兼容端点, 适配任意 SDK" }
        Section { title: "端点", count: None,
            div { class: "divide-y divide-line border-y border-line",
                EndpointRow { method: "GET", path: "/v1/models", desc: "可用模型" }
                EndpointRow { method: "POST", path: "/v1/chat/completions", desc: "OpenAI 对话, 支持流式" }
                EndpointRow { method: "POST", path: "/v1/messages", desc: "Anthropic 消息, 支持流式" }
                EndpointRow { method: "POST", path: "/v1/messages/count_tokens", desc: "token 计数" }
                EndpointRow { method: "GET", path: "/healthz", desc: "健康检查, 免鉴权" }
            }
        }
        Section { title: "示例", count: None,
            div { class: "space-y-4",
                CodeBlock { id: "curl-chat", title: "OpenAI 非流式".to_string(), copied,
                    code: format!("curl {base}/v1/chat/completions \\\n  -H \"content-type: application/json\" \\\n  -d '{{\"model\":\"deepseek/deepseek-v4-flash\",\"messages\":[{{\"role\":\"user\",\"content\":\"hello\"}}]}}'") }
                CodeBlock { id: "curl-stream", title: "OpenAI 流式".to_string(), copied,
                    code: format!("curl {base}/v1/chat/completions \\\n  -H \"content-type: application/json\" \\\n  -d '{{\"model\":\"mimo/mimo-v2.5\",\"stream\":true,\"messages\":[{{\"role\":\"user\",\"content\":\"hello\"}}]}}'") }
                CodeBlock { id: "curl-anthropic", title: "Anthropic 消息".to_string(), copied,
                    code: format!("curl {base}/v1/messages \\\n  -H \"x-api-key: <key>\" -H \"content-type: application/json\" \\\n  -d '{{\"model\":\"deepseek/deepseek-v4-flash\",\"max_tokens\":1024,\"messages\":[{{\"role\":\"user\",\"content\":\"hello\"}}]}}'") }
            }
        }
    }
}

#[component]
fn EndpointRow(method: &'static str, path: &'static str, desc: &'static str) -> Element {
    let tone = if method == "GET" {
        "text-alive"
    } else {
        "text-sky-700"
    };
    rsx! {
        div { class: "flex h-10 items-center gap-4 px-1",
            span { class: "w-10 shrink-0 text-xs font-semibold {tone}", {method} }
            Data { text: path.to_string(), class: "text-sm".to_string() }
            span { class: "ml-auto text-xs text-ink/45", {desc} }
        }
    }
}

#[component]
fn CodeBlock(id: &'static str, title: String, mut copied: Signal<Option<&'static str>>, code: String) -> Element {
    let code_display = code.clone();
    rsx! {
        div {
            div { class: "flex items-baseline justify-between",
                span { class: "text-xs text-ink/55", {title} }
                Button { variant: ButtonVariant::Ghost, class: "h-6 rounded-sm text-xs",
                    on_click: move |_| {
                        let text = code.clone();
                        spawn(async move {
                            copy_to_clipboard(&text);
                            copied.set(Some(id));
                        });
                    },
                    if copied() == Some(id) { "已复制" } else { "复制" } }
            }
            pre { class: "mt-1 overflow-x-auto rounded-sm bg-ink px-4 py-3 font-data text-xs leading-relaxed text-paper", {code_display} }
        }
    }
}

fn copy_to_clipboard(text: &str) {
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "navigator", "clipboard"])]
        fn write_text(text: &str);
    }
    write_text(text);
}

fn mask_key(k: &str) -> String {
    if k.len() <= 10 { k.to_string() } else { format!("{}...{}", &k[..8], &k[k.len() - 4..]) }
}

use lucide_dioxus::{Plus, Upload};
use lumen_blocks::components::button::{Button, ButtonVariant};
use lumen_blocks::components::input::Input;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { Router::<Route> {} }
}


// ---------------------------------------------------------------------------
// 用量
// ---------------------------------------------------------------------------

#[component]
fn Usage() -> Element {
    let mut summary: Signal<Value> = use_signal(|| Value::Null);
    let mut recent: Signal<Value> = use_signal(|| Value::Null);

    use_future(move || async move {
        if let Ok(v) = api_get("/admin/usage/summary?hours=24").await {
            summary.set(v);
        }
        if let Ok(v) = api_get("/admin/usage/recent?limit=30").await {
            recent.set(v);
        }
    });

    let s = summary();
    let fmt_ts = |ts: i64| -> String {
        // 相对分钟
        let now = js_sys::Date::now() as i64;
        let mins = (now - ts).max(0) / 60000;
        if mins < 1 { "刚刚".into() } else { format!("{mins} 分钟前") }
    };

    rsx! {
        PageHeader { title: "用量".to_string(), desc: "请求 / token / 延迟 / 错误 — SQLite 持久化".to_string() }
        div { class: "grid gap-3 sm:grid-cols-2 xl:grid-cols-4",
            {[
                ("请求数", s["requests"].as_i64().unwrap_or(0).to_string()),
                ("错误数", s["errors"].as_i64().unwrap_or(0).to_string()),
                ("Token (入/出)", format!("{} / {}", s["prompt_tokens"].as_i64().unwrap_or(0), s["completion_tokens"].as_i64().unwrap_or(0))),
                ("延迟 p50 / p95", format!("{}ms / {}ms", s["latency_p50_ms"].as_i64().unwrap_or(0), s["latency_p95_ms"].as_i64().unwrap_or(0))),
            ].into_iter().map(|(label, value)| rsx! {
                div { key: "{label}", class: "border border-line bg-white px-4 py-3",
                    div { class: "text-xs text-ink/50", {label} }
                    div { class: "mt-1 text-xl font-semibold tracking-tight text-ink", {value} }
                }
            })}
        }
        Section { title: "按模型分布", count: Some(s["by_model"].as_array().map(|a| a.len()).unwrap_or(0)),
            if s["by_model"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
                EmptyRow { text: "暂无数据 — 发起一次对话后出现" }
            }
            for m in s["by_model"].as_array().cloned().unwrap_or_default() {
                Row {
                    left: rsx! { Data { text: m["model"].as_str().unwrap_or("").to_string(), class: "text-sm".to_string() } },
                    right: rsx! { span { class: "text-sm text-ink/55", {format!("{} 次", m["requests"].as_i64().unwrap_or(0))} } },
                }
            }
        }
        Section { title: "最近请求", count: Some(recent()["items"].as_array().map(|a| a.len()).unwrap_or(0)),
            if recent()["items"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
                EmptyRow { text: "暂无记录" }
            }
            for it in recent()["items"].as_array().cloned().unwrap_or_default() {
                Row {
                    left: rsx! {
                        div { class: "flex min-w-0 items-center gap-2.5",
                            AliveDot { alive: (200..300).contains(&it["status"].as_i64().unwrap_or(0)) }
                            span { class: "truncate text-sm text-ink", {it["model"].as_str().unwrap_or("").to_string()} }
                        }
                    },
                    right: rsx! {
                        div { class: "flex items-center gap-3 text-xs text-ink/55",
                            span { {format!("{}+{}", it["prompt_tokens"].as_i64().unwrap_or(0), it["completion_tokens"].as_i64().unwrap_or(0))} }
                            span { {format!("{}ms", it["latency_ms"].as_i64().unwrap_or(0))} }
                            span { class: "text-ink/40", {fmt_ts(it["ts"].as_i64().unwrap_or(0))} }
                        }
                    },
                }
            }
        }
    }
}


// ---------------------------------------------------------------------------
// 实时日志 (SSE)
// ---------------------------------------------------------------------------

#[component]
fn Logs() -> Element {
    let mut lines: Signal<Vec<String>> = use_signal(Vec::new);
    let mut connected = use_signal(|| false);
    let mut logbox = use_signal(String::new);

    rsx! {
        PageHeader { title: "日志".to_string(), desc: "网关实时事件 — SSE 流式推送".to_string() }
        div { class: "flex items-center gap-2",
            Button { variant: if *connected.read() { ButtonVariant::Outline } else { ButtonVariant::Primary }, class: "h-8 rounded-sm",
                on_click: move |_| {
                    connected.set(true);
                    let ws = web_sys::window().unwrap();
                    let es = web_sys::EventSource::new("http://127.0.0.1:8787/admin/logs/stream").unwrap();
                    let setter = move |e: web_sys::MessageEvent| {
                        let text = e.data().as_string().unwrap_or_default();
                        lines.push(text);
                        if lines.len() > 300 {
                            let cur = lines();
                            lines.set(cur.split_off(cur.len() - 300));
                        }
                        logbox.set(format!("{:?}", lines.len()));
                    };
                    let cb = wasm_bindgen::closure::Closure::wrap(Box::new(setter) as Box<dyn FnMut(web_sys::MessageEvent)>);
                    es.set_onmessage(Some(cb.as_ref().unchecked_ref()));
                    cb.forget();
                },
                if *connected.read() { "已连接" } else { "连接" } }
            span { class: "text-xs text-ink/45", "接入后自动滚动, 保留最近 300 条" }
        }
        Section { title: "事件流", count: Some(lines().len()),
            if lines().is_empty() {
                EmptyRow { text: "暂无事件 — 点「连接」开始接收; 或发起一次对话" }
            }
            for (i, l) in lines().iter().enumerate().rev().take(80) {
                div { key: "{i}-{l}", class: "border-b border-line px-4 py-1.5 font-mono text-xs text-ink/75",
                    {l.clone()} }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// API 测试 (Playground)
// ---------------------------------------------------------------------------

#[component]
fn Playground() -> Element {
    let mut model = use_signal(|| "z-ai/glm-5.3-flash".to_string());
    let mut prompt = use_signal(|| "你好, 介绍一下你自己".to_string());
    let mut output = use_signal(String::new);
    let mut running = use_signal(|| false);
    let models = ["z-ai/glm-5.3-flash", "deepseek/deepseek-v4-flash", "mimo/mimo-v2.5", "upstage/solar-pro4"];

    let run = move |_| {
        if *running.read() { return; }
        running.set(true);
        output.set(String::new());
        spawn(async move {
            let body = serde_json::json!({
                "model": model(),
                "messages": [{"role": "user", "content": prompt()}],
                "stream": true,
            });
            let opts = web_sys::RequestInit::new();
            opts.set_method("POST");
            opts.set_body(body.to_string().as_str());
            let req = web_sys::Request::new_with_str_and_init("http://127.0.0.1:8787/v1/chat/completions", &opts).unwrap();
            req.headers().set("authorization", "Bearer sk-test").ok();
            req.headers().set("content-type", "application/json").ok();
            match web_sys::window().unwrap().fetch_with_request(&req).call() {
                Ok(resp) => {
                    let resp: web_sys::Response = resp.into().into();
                    if let Ok(text) = js_sys::Promise::from(resp.text().unwrap()).await {
                        let full = text.as_string().unwrap_or_default();
                        // SSE 行解析
                        let mut acc = String::new();
                        for line in full.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" { break; }
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                                    if let Some(c) = v["choices"][0]["delta"]["content"].as_str() {
                                        acc.push_str(c);
                                        output.set(acc.clone());
                                    }
                                }
                            }
                        }
                        if acc.is_empty() {
                            output.set(full.clone());
                        }
                    }
                }
                Err(e) => output.set(format!("请求失败: {e:?}")),
            }
            running.set(false);
        });
    };

    rsx! {
        PageHeader { title: "测试".to_string(), desc: "面板内直接发起对话, 验证网关与账号链路".to_string() }
        Section { title: "请求", count: None,
            div { class: "flex flex-wrap gap-2 px-4 py-3",
                for m in models {
                    button {
                        key: "{m}",
                        class: if model() == m { "h-8 rounded-sm border-2 border-ink bg-ink/[0.06] px-3 text-xs font-medium text-ink" } else { "h-8 rounded-sm border border-line px-3 text-xs text-ink/55" },
                        onclick: move |_| model.set(m.to_string()),
                        {m.rsplit('/').next().unwrap_or(m)} }
                }
            }
            div { class: "px-4 pb-4",
                textarea {
                    class: "min-h-20 w-full border border-line px-3 py-2 text-sm text-ink focus:border-ink focus:outline-none",
                    value: prompt(),
                    oninput: move |e| prompt.set(e.value()),
                }
                div { class: "mt-3 flex items-center gap-3",
                    Button { variant: ButtonVariant::Primary, class: "h-9 rounded-sm", disabled: running(),
                        on_click: run,
                        if running() { "生成中…" } else { "发送" } }
                    span { class: "text-xs text-ink/45", "走网关 /v1/chat/completions (流式)" }
                }
            }
        }
        Section { title: "输出", count: None,
            div { class: "min-h-24 whitespace-pre-wrap px-4 py-3 text-sm text-ink",
                if output().is_empty() { span { class: "text-ink/35", "等待发送…" } } else { {output()} }
            }
        }
    }
}
