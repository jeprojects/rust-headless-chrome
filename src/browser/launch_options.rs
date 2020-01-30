#[cfg(feature = "fetch")]
use super::fetcher::FetcherOptions;
#[cfg(not(feature = "pipe"))]
use std::collections::HashMap;
use std::ffi::OsStr;
use std::time::Duration;

/// Represents the way in which Chrome is run. By default it will search for a Chrome
/// binary on the system, use an available port for debugging, and start in headless mode.
#[derive(Builder)]
pub struct LaunchOptions<'a> {
    /// Determintes whether to run headless version of the browser. Defaults to true.
    #[builder(default = "true")]
    pub(crate) headless: bool,
    /// Determines whether to run the browser with a sandbox.
    #[builder(default = "true")]
    pub(crate) sandbox: bool,
    /// Launch the browser with a specific window width and height.
    #[builder(default = "None")]
    pub(crate) window_size: Option<(u32, u32)>,
    /// Launch the browser with a specific debugging port.
    #[cfg(not(feature = "pipe"))]
    #[builder(default = "None")]
    pub(crate) port: Option<u16>,

    /// Path for Chrome or Chromium.
    ///
    /// If unspecified, the create will try to automatically detect a suitable binary.
    #[builder(default = "None")]
    pub(crate) path: Option<std::path::PathBuf>,

    /// A list of Chrome extensions to load.
    ///
    /// An extension should be a path to a folder containing the extension code.
    /// CRX files cannot be used directly and must be first extracted.
    ///
    /// Note that Chrome does not support loading extensions in headless-mode.
    /// See https://bugs.chromium.org/p/chromium/issues/detail?id=706008#c5
    #[builder(default)]
    pub(crate) extensions: Vec<&'a OsStr>,

    /// Additional arguments to pass to the browser instance. The list of Chromium
    /// flags can be found: http://peter.sh/experiments/chromium-command-line-switches/.
    #[builder(default)]
    pub(crate) args: Vec<&'a OsStr>,

    /// The options to use for fetching a version of chrome when `path` is None.
    ///
    /// By default, we'll use a revision guaranteed to work with our API and will
    /// download and install that revision of chrome the first time a Process is created.
    #[cfg(feature = "fetch")]
    #[builder(default)]
    pub(crate) fetcher_options: FetcherOptions,

    /// How long to keep the WebSocket to the browser for after not receiving any events from it
    /// Defaults to 30 seconds
    #[builder(default = "Duration::from_secs(300)")]
    pub idle_browser_timeout: Duration,

    /// Environment variables to set for the Chromium process.
    /// Passes value through to std::process::Command::envs.
    #[cfg(not(feature = "pipe"))]
    #[builder(default = "None")]
    pub process_envs: Option<HashMap<String, String>>,
}

impl<'a> LaunchOptions<'a> {
    pub fn default_builder() -> LaunchOptionsBuilder<'a> {
        LaunchOptionsBuilder::default()
    }
}

/// These are passed to the Chrome binary by default.
/// Via https://github.com/GoogleChrome/puppeteer/blob/master/lib/Launcher.js#L38
pub(crate) static DEFAULT_ARGS: [&str; 23] = [
    "--disable-background-networking",
    "--enable-features=NetworkService,NetworkServiceInProcess",
    "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows",
    "--disable-breakpad",
    "--disable-client-side-phishing-detection",
    "--disable-component-extensions-with-background-pages",
    "--disable-default-apps",
    "--disable-dev-shm-usage",
    "--disable-extensions",
    // BlinkGenPropertyTrees disabled due to crbug.com/937609
    "--disable-features=TranslateUI,BlinkGenPropertyTrees",
    "--disable-hang-monitor",
    "--disable-ipc-flooding-protection",
    "--disable-popup-blocking",
    "--disable-prompt-on-repost",
    "--disable-renderer-backgrounding",
    "--disable-sync",
    "--force-color-profile=srgb",
    "--metrics-recording-only",
    "--no-first-run",
    "--enable-automation",
    "--password-store=basic",
    "--use-mock-keychain",
];
