use crate::{
    CacheLayerPolicy, MokaCacheOptions, RevalidationPolicy, ServerPrivatePolicy,
    ServerRenderPolicy, WebRouteMode,
};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ServerRuntimeConfig {
    pub default_locale: String,
    pub default_route_mode: Option<WebRouteMode>,
    pub render_pass_limit: Option<usize>,
    pub cache: ServerCacheConfig,
    pub http: ServerHttpConfig,
    pub sessions: ServerSessionConfig,
    pub workers: ServerBrowserArtifactConfig,
    pub islands: ServerIslandConfig,
}

impl Default for ServerRuntimeConfig {
    fn default() -> Self {
        Self {
            default_locale: "en".to_string(),
            default_route_mode: None,
            render_pass_limit: None,
            cache: ServerCacheConfig::default(),
            http: ServerHttpConfig::default(),
            sessions: ServerSessionConfig::default(),
            workers: ServerBrowserArtifactConfig::default(),
            islands: ServerIslandConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServerCacheConfig {
    pub provider: ServerCacheProvider,
    pub moka: MokaCacheOptions,
    pub ttl: Option<Duration>,
    pub stale_while_revalidate: Option<Duration>,
    pub redis_url: Option<String>,
    pub redis_url_env: Option<String>,
    pub redis_prefix: Option<String>,
    pub layers: Vec<ServerCacheLayerConfig>,
}

impl Default for ServerCacheConfig {
    fn default() -> Self {
        Self {
            provider: ServerCacheProvider::Moka,
            moka: MokaCacheOptions::default(),
            ttl: None,
            stale_while_revalidate: None,
            redis_url: None,
            redis_url_env: None,
            redis_prefix: None,
            layers: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServerCacheProvider {
    #[default]
    Moka,
    Redis,
    Pipeline,
}

#[derive(Clone, Debug)]
pub struct ServerCacheLayerConfig {
    pub name: String,
    pub provider: ServerCacheProvider,
    pub policy: CacheLayerPolicy,
    pub moka: MokaCacheOptions,
    pub redis_url: Option<String>,
    pub redis_url_env: Option<String>,
    pub redis_prefix: Option<String>,
    pub ttl: Option<Duration>,
    pub stale_while_revalidate: Option<Duration>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerHttpConfig {
    pub base_url: Option<String>,
    pub trust_proxy_headers: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerSessionConfig {
    pub provider: ServerSessionProvider,
    pub cookie_name: String,
    pub signing_key_env: Option<String>,
    pub secure: bool,
    pub same_site: ServerSameSite,
}

impl Default for ServerSessionConfig {
    fn default() -> Self {
        Self {
            provider: ServerSessionProvider::Cookie,
            cookie_name: crate::render::DEFAULT_SESSION_COOKIE_NAME.to_string(),
            signing_key_env: None,
            secure: false,
            same_site: ServerSameSite::Lax,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServerSessionProvider {
    #[default]
    Cookie,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServerSameSite {
    Strict,
    #[default]
    Lax,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerBrowserArtifactConfig {
    pub separate_artifacts: bool,
    pub bridge: ServerWorkerBridge,
}

impl Default for ServerBrowserArtifactConfig {
    fn default() -> Self {
        Self {
            separate_artifacts: true,
            bridge: ServerWorkerBridge::Generated,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerIslandConfig {
    pub separate_artifacts: bool,
    pub preload: ServerIslandPreload,
}

impl Default for ServerIslandConfig {
    fn default() -> Self {
        Self {
            separate_artifacts: true,
            preload: ServerIslandPreload::Route,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServerWorkerBridge {
    #[default]
    Generated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServerIslandPreload {
    None,
    #[default]
    Route,
}

#[derive(Debug, Deserialize, Default)]
struct FissionToml {
    server: Option<ServerToml>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerToml {
    default_locale: Option<String>,
    default_route_mode: Option<String>,
    render_pass_limit: Option<usize>,
    http: Option<ServerHttpToml>,
    cache: Option<ServerCacheToml>,
    sessions: Option<ServerSessionToml>,
    workers: Option<ServerWorkersToml>,
    islands: Option<ServerIslandsToml>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerCacheToml {
    provider: Option<String>,
    max_capacity: Option<u64>,
    ttl: Option<String>,
    default_ttl: Option<String>,
    stale_while_revalidate: Option<String>,
    redis_url: Option<String>,
    url: Option<String>,
    url_env: Option<String>,
    redis_prefix: Option<String>,
    prefix: Option<String>,
    layers: Option<Vec<ServerCacheLayerToml>>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerCacheLayerToml {
    name: Option<String>,
    provider: Option<String>,
    policy: Option<String>,
    max_capacity: Option<u64>,
    ttl: Option<String>,
    stale_while_revalidate: Option<String>,
    redis_url: Option<String>,
    url: Option<String>,
    url_env: Option<String>,
    redis_prefix: Option<String>,
    prefix: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerHttpToml {
    base_url: Option<String>,
    trust_proxy_headers: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerSessionToml {
    provider: Option<String>,
    cookie_name: Option<String>,
    signing_key_env: Option<String>,
    secure: Option<bool>,
    same_site: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerWorkersToml {
    separate_artifacts: Option<bool>,
    bridge: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerIslandsToml {
    separate_artifacts: Option<bool>,
    preload: Option<String>,
}

impl ServerRuntimeConfig {
    pub fn load(project_dir: &Path) -> Result<Self> {
        let path = project_dir.join("fission.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let root: FissionToml =
            toml::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))?;
        Self::from_toml(root)
    }

    fn from_toml(root: FissionToml) -> Result<Self> {
        let Some(server) = root.server else {
            return Ok(Self::default());
        };
        let mut cache = ServerCacheConfig::default();
        if let Some(cache_toml) = server.cache {
            cache = ServerCacheConfig::from_toml(cache_toml)?;
        }
        Ok(Self {
            default_locale: server.default_locale.unwrap_or_else(|| "en".to_string()),
            default_route_mode: server
                .default_route_mode
                .as_deref()
                .map(|mode| parse_route_mode(mode, &cache))
                .transpose()?,
            render_pass_limit: server.render_pass_limit,
            cache,
            http: server
                .http
                .map(ServerHttpConfig::from_toml)
                .transpose()?
                .unwrap_or_default(),
            sessions: server
                .sessions
                .map(ServerSessionConfig::from_toml)
                .transpose()?
                .unwrap_or_default(),
            workers: server
                .workers
                .map(ServerBrowserArtifactConfig::from_toml)
                .transpose()?
                .unwrap_or_default(),
            islands: server
                .islands
                .map(ServerIslandConfig::from_toml)
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl ServerCacheConfig {
    fn from_toml(value: ServerCacheToml) -> Result<Self> {
        let provider = parse_cache_provider(value.provider.as_deref().unwrap_or("moka"))?;
        let ttl_value = value.ttl.as_deref().or(value.default_ttl.as_deref());
        let ttl = ttl_value.map(parse_duration).transpose()?;
        let stale_while_revalidate = value
            .stale_while_revalidate
            .as_deref()
            .map(parse_duration)
            .transpose()?;
        Ok(Self {
            provider,
            moka: MokaCacheOptions {
                max_capacity: value
                    .max_capacity
                    .unwrap_or_else(|| MokaCacheOptions::default().max_capacity),
            },
            ttl,
            stale_while_revalidate,
            redis_url: value.redis_url.or(value.url),
            redis_url_env: value.url_env,
            redis_prefix: value.redis_prefix.or(value.prefix),
            layers: value
                .layers
                .unwrap_or_default()
                .into_iter()
                .enumerate()
                .map(|(index, layer)| ServerCacheLayerConfig::from_toml(index, layer))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl ServerCacheLayerConfig {
    fn from_toml(index: usize, value: ServerCacheLayerToml) -> Result<Self> {
        Ok(Self {
            name: value.name.unwrap_or_else(|| format!("layer-{index}")),
            provider: parse_cache_provider(value.provider.as_deref().unwrap_or("moka"))?,
            policy: parse_cache_layer_policy(value.policy.as_deref().unwrap_or("write-through"))?,
            moka: MokaCacheOptions {
                max_capacity: value
                    .max_capacity
                    .unwrap_or_else(|| MokaCacheOptions::default().max_capacity),
            },
            redis_url: value.redis_url.or(value.url),
            redis_url_env: value.url_env,
            redis_prefix: value.redis_prefix.or(value.prefix),
            ttl: value.ttl.as_deref().map(parse_duration).transpose()?,
            stale_while_revalidate: value
                .stale_while_revalidate
                .as_deref()
                .map(parse_duration)
                .transpose()?,
        })
    }
}

impl ServerHttpConfig {
    fn from_toml(value: ServerHttpToml) -> Result<Self> {
        Ok(Self {
            base_url: value.base_url.map(normalize_base_url).transpose()?,
            trust_proxy_headers: value.trust_proxy_headers.unwrap_or(false),
        })
    }
}

impl ServerSessionConfig {
    fn from_toml(value: ServerSessionToml) -> Result<Self> {
        let config = Self {
            provider: parse_session_provider(value.provider.as_deref().unwrap_or("cookie"))?,
            cookie_name: value
                .cookie_name
                .unwrap_or_else(|| crate::render::DEFAULT_SESSION_COOKIE_NAME.to_string()),
            signing_key_env: value.signing_key_env,
            secure: value.secure.unwrap_or(false),
            same_site: parse_same_site(value.same_site.as_deref().unwrap_or("lax"))?,
        };
        if config.same_site == ServerSameSite::None && !config.secure {
            bail!("[server.sessions].same_site = \"none\" requires secure = true");
        }
        if !safe_cookie_name(&config.cookie_name) {
            bail!(
                "[server.sessions].cookie_name `{}` is not a valid cookie name",
                config.cookie_name
            );
        }
        Ok(config)
    }
}

impl ServerBrowserArtifactConfig {
    fn from_toml(value: ServerWorkersToml) -> Result<Self> {
        let bridge = match normalize_token(value.bridge.as_deref().unwrap_or("generated")).as_str()
        {
            "generated" => ServerWorkerBridge::Generated,
            other => bail!("unsupported [server.workers].bridge `{other}`"),
        };
        Ok(Self {
            separate_artifacts: value.separate_artifacts.unwrap_or(true),
            bridge,
        })
    }
}

impl ServerIslandConfig {
    fn from_toml(value: ServerIslandsToml) -> Result<Self> {
        let preload = match normalize_token(value.preload.as_deref().unwrap_or("route")).as_str() {
            "none" | "off" => ServerIslandPreload::None,
            "route" => ServerIslandPreload::Route,
            other => bail!("unsupported [server.islands].preload `{other}`"),
        };
        Ok(Self {
            separate_artifacts: value.separate_artifacts.unwrap_or(true),
            preload,
        })
    }
}

fn parse_cache_provider(value: &str) -> Result<ServerCacheProvider> {
    match normalize_token(value).as_str() {
        "moka" => Ok(ServerCacheProvider::Moka),
        "redis" => Ok(ServerCacheProvider::Redis),
        "pipeline" => Ok(ServerCacheProvider::Pipeline),
        other => bail!("unsupported [server.cache].provider `{other}`"),
    }
}

fn parse_cache_layer_policy(value: &str) -> Result<CacheLayerPolicy> {
    match normalize_token(value).as_str() {
        "writethrough" | "write" => Ok(CacheLayerPolicy::WriteThrough),
        "readonly" | "read" => Ok(CacheLayerPolicy::ReadOnly),
        "hotonly" | "hot" => Ok(CacheLayerPolicy::HotOnly),
        other => bail!("unsupported server cache layer policy `{other}`"),
    }
}

fn parse_session_provider(value: &str) -> Result<ServerSessionProvider> {
    match normalize_token(value).as_str() {
        "cookie" => Ok(ServerSessionProvider::Cookie),
        other => bail!("unsupported [server.sessions].provider `{other}`"),
    }
}

fn parse_same_site(value: &str) -> Result<ServerSameSite> {
    match normalize_token(value).as_str() {
        "strict" => Ok(ServerSameSite::Strict),
        "lax" => Ok(ServerSameSite::Lax),
        "none" => Ok(ServerSameSite::None),
        other => bail!("unsupported [server.sessions].same_site `{other}`"),
    }
}

fn normalize_base_url(value: String) -> Result<String> {
    let trimmed = value.trim().trim_end_matches('/').to_string();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Ok(trimmed)
    } else {
        bail!("[server.http].base_url must start with http:// or https://")
    }
}

fn parse_route_mode(value: &str, cache: &ServerCacheConfig) -> Result<WebRouteMode> {
    match normalize_token(value).as_str() {
        "static" => Ok(WebRouteMode::Static),
        "server" | "dynamic" => Ok(WebRouteMode::Server(ServerRenderPolicy::default())),
        "serverprivate" | "private" | "privatesession" | "session" => {
            Ok(WebRouteMode::ServerPrivate(ServerPrivatePolicy::default()))
        }
        "clientapp" | "client" => Ok(WebRouteMode::ClientApp(Default::default())),
        "revalidated" => {
            let mut policy = RevalidationPolicy::new(cache.ttl.unwrap_or(Duration::from_secs(60)));
            if let Some(stale) = cache.stale_while_revalidate {
                policy = policy.stale_while_revalidate(stale);
            }
            Ok(WebRouteMode::Revalidated(policy))
        }
        other => bail!("unsupported [server].default_route_mode `{other}`"),
    }
}

fn parse_duration(value: &str) -> Result<Duration> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("duration must not be empty");
    }
    let split_at = trimmed
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let (number, unit) = trimmed.split_at(split_at);
    let amount: u64 = number
        .parse()
        .with_context(|| format!("duration `{value}` must start with an integer"))?;
    match unit.trim() {
        "" | "s" | "sec" | "secs" | "second" | "seconds" => Ok(Duration::from_secs(amount)),
        "ms" | "millisecond" | "milliseconds" => Ok(Duration::from_millis(amount)),
        "m" | "min" | "mins" | "minute" | "minutes" => Ok(Duration::from_secs(amount * 60)),
        "h" | "hr" | "hrs" | "hour" | "hours" => Ok(Duration::from_secs(amount * 60 * 60)),
        "d" | "day" | "days" => Ok(Duration::from_secs(amount * 60 * 60 * 24)),
        other => bail!("unsupported duration unit `{other}` in `{value}`"),
    }
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, '-' | '_' | ' '))
        .flat_map(char::to_lowercase)
        .collect()
}

fn safe_cookie_name(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_server_cache_and_default_route_mode() {
        let config = ServerRuntimeConfig::from_toml(FissionToml {
            server: Some(ServerToml {
                default_route_mode: Some("revalidated".to_string()),
                render_pass_limit: Some(7),
                cache: Some(ServerCacheToml {
                    provider: Some("moka".to_string()),
                    max_capacity: Some(42),
                    ttl: Some("5m".to_string()),
                    default_ttl: None,
                    stale_while_revalidate: Some("30s".to_string()),
                    redis_url: None,
                    url: None,
                    url_env: None,
                    redis_prefix: None,
                    prefix: None,
                    layers: None,
                }),
                ..Default::default()
            }),
        })
        .unwrap();

        assert_eq!(config.render_pass_limit, Some(7));
        assert_eq!(config.cache.moka.max_capacity, 42);
        assert!(matches!(
            config.default_route_mode,
            Some(WebRouteMode::Revalidated(_))
        ));
        let Some(WebRouteMode::Revalidated(policy)) = config.default_route_mode else {
            unreachable!();
        };
        assert_eq!(policy.ttl, Duration::from_secs(300));
        assert_eq!(policy.stale_while_revalidate, Some(Duration::from_secs(30)));
    }

    #[test]
    fn rejects_unknown_server_config_values() {
        let error = parse_route_mode("whatever", &ServerCacheConfig::default()).unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported [server].default_route_mode"));

        let error = parse_duration("10fortnights").unwrap_err();
        assert!(error.to_string().contains("unsupported duration unit"));
    }

    #[test]
    fn parses_server_http_sessions_workers_islands_and_cache_layers() {
        let config = ServerRuntimeConfig::from_toml(FissionToml {
            server: Some(ServerToml {
                default_locale: Some("en-GB".to_string()),
                http: Some(ServerHttpToml {
                    base_url: Some("https://shop.example/".to_string()),
                    trust_proxy_headers: Some(true),
                }),
                sessions: Some(ServerSessionToml {
                    provider: Some("cookie".to_string()),
                    cookie_name: Some("shop_session".to_string()),
                    signing_key_env: Some("SHOP_SESSION_KEY".to_string()),
                    secure: Some(true),
                    same_site: Some("none".to_string()),
                }),
                cache: Some(ServerCacheToml {
                    provider: Some("pipeline".to_string()),
                    layers: Some(vec![
                        ServerCacheLayerToml {
                            name: Some("hot".to_string()),
                            provider: Some("moka".to_string()),
                            policy: Some("hot-only".to_string()),
                            max_capacity: Some(256),
                            ..Default::default()
                        },
                        ServerCacheLayerToml {
                            name: Some("shared".to_string()),
                            provider: Some("redis".to_string()),
                            policy: Some("write-through".to_string()),
                            url_env: Some("REDIS_URL".to_string()),
                            prefix: Some("shop".to_string()),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                }),
                workers: Some(ServerWorkersToml {
                    separate_artifacts: Some(true),
                    bridge: Some("generated".to_string()),
                }),
                islands: Some(ServerIslandsToml {
                    separate_artifacts: Some(true),
                    preload: Some("none".to_string()),
                }),
                ..Default::default()
            }),
        })
        .unwrap();

        assert_eq!(config.default_locale, "en-GB");
        assert_eq!(
            config.http.base_url.as_deref(),
            Some("https://shop.example")
        );
        assert!(config.http.trust_proxy_headers);
        assert_eq!(config.sessions.cookie_name, "shop_session");
        assert_eq!(
            config.sessions.signing_key_env.as_deref(),
            Some("SHOP_SESSION_KEY")
        );
        assert_eq!(config.sessions.same_site, ServerSameSite::None);
        assert!(matches!(
            config.cache.provider,
            ServerCacheProvider::Pipeline
        ));
        assert_eq!(config.cache.layers.len(), 2);
        assert_eq!(config.cache.layers[0].policy, CacheLayerPolicy::HotOnly);
        assert_eq!(
            config.cache.layers[1].redis_url_env.as_deref(),
            Some("REDIS_URL")
        );
        assert_eq!(config.cache.layers[1].redis_prefix.as_deref(), Some("shop"));
        assert_eq!(config.islands.preload, ServerIslandPreload::None);
    }

    #[test]
    fn rejects_insecure_same_site_none_session_cookie() {
        let error = ServerSessionConfig::from_toml(ServerSessionToml {
            secure: Some(false),
            same_site: Some("none".to_string()),
            ..Default::default()
        })
        .unwrap_err();

        assert!(error.to_string().contains("requires secure = true"));
    }
}
