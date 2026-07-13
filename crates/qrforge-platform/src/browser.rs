use qrforge_application::{BrowserPort, PortError};
use qrforge_domain::SafeHttpUrl;

/// System-default browser adapter.
///
/// The safe adapter API accepts only URLs already validated by the domain. It
/// does not expose arbitrary paths, commands, arguments, or shell execution.
#[derive(Default)]
pub struct SystemBrowser;

impl BrowserPort for SystemBrowser {
    fn open(&self, url: &SafeHttpUrl) -> Result<(), PortError> {
        webbrowser::open(url.as_str())
            .map_err(|error| PortError::new("browser_open", error.to_string()))
    }
}
