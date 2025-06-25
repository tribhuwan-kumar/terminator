use crate::element::UIElement;
use crate::exceptions::automation_error_to_pyerr;
use crate::locator::Locator;
use crate::types::{CommandOutput, ScreenshotResult};
use ::terminator_core::Desktop as TerminatorDesktop;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio as pyo3_tokio;
use pyo3_async_runtimes::TaskLocals;
use pyo3_stub_gen::derive::*;
use std::sync::Once;

/// Main entry point for desktop automation.
#[gen_stub_pyclass]
#[pyclass(name = "Desktop")]
pub struct Desktop {
    inner: TerminatorDesktop,
}

#[gen_stub_pymethods]
#[pymethods]
impl Desktop {
    #[new]
    #[pyo3(signature = (use_background_apps=None, activate_app=None, log_level=None))]
    #[pyo3(text_signature = "(use_background_apps=False, activate_app=False, log_level=None)")]
    /// Create a new Desktop automation instance with configurable options.
    ///
    /// Args:
    ///     use_background_apps (bool, optional): Enable background apps support. Defaults to False.
    ///     activate_app (bool, optional): Enable app activation support. Defaults to False.
    ///     log_level (str, optional): Logging level (e.g., 'info', 'debug', 'warn', 'error'). Defaults to 'info'.
    ///
    /// Returns:
    ///     Desktop: A new Desktop automation instance.
    pub fn new(
        use_background_apps: Option<bool>,
        activate_app: Option<bool>,
        log_level: Option<String>,
    ) -> PyResult<Self> {
        static INIT: Once = Once::new();
        let log_level = log_level.unwrap_or_else(|| "info".to_string());
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(log_level)
                .try_init();
        });
        let use_background_apps = use_background_apps.unwrap_or(false);
        let activate_app = activate_app.unwrap_or(false);
        let desktop = TerminatorDesktop::new(use_background_apps, activate_app)
            .map_err(automation_error_to_pyerr)?;
        Ok(Desktop { inner: desktop })
    }

    #[pyo3(text_signature = "($self)")]
    /// Get the root UI element of the desktop.
    ///
    /// Returns:
    ///     UIElement: The root UI element.
    pub fn root(&self) -> PyResult<UIElement> {
        let root = self.inner.root();
        Ok(UIElement { inner: root })
    }

    #[pyo3(text_signature = "($self)")]
    /// Get a list of all running applications.
    ///
    /// Returns:
    ///     List[UIElement]: List of application UI elements.
    pub fn applications(&self) -> PyResult<Vec<UIElement>> {
        self.inner
            .applications()
            .map(|apps| apps.into_iter().map(|e| UIElement { inner: e }).collect())
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(text_signature = "($self, name)")]
    /// Get a running application by name.
    ///
    /// Args:
    ///     name (str): The name of the application to find.
    ///
    /// Returns:
    ///     UIElement: The application UI element.
    pub fn application(&self, name: &str) -> PyResult<UIElement> {
        self.inner
            .application(name)
            .map(|e| UIElement { inner: e })
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(name = "open_application", text_signature = "($self, name)")]
    /// Open an application by name.
    ///
    /// Args:
    ///     name (str): The name of the application to open.
    pub fn open_application(&self, name: &str) -> PyResult<UIElement> {
        self.inner
            .open_application(name)
            .map(|e| UIElement { inner: e })
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(name = "activate_application", text_signature = "($self, name)")]
    /// Activate an application by name.
    ///
    /// Args:
    ///     name (str): The name of the application to activate.
    pub fn activate_application(&self, name: &str) -> PyResult<()> {
        self.inner
            .activate_application(name)
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(name = "locator", text_signature = "($self, selector)")]
    /// Create a locator for finding UI elements.
    ///
    /// Args:
    ///     selector (str): The selector string to find elements.
    ///
    /// Returns:
    ///     Locator: A locator for finding elements.
    pub fn locator(&self, selector: &str) -> PyResult<Locator> {
        let locator = self.inner.locator(selector);
        Ok(Locator { inner: locator })
    }

    #[pyo3(name = "run_command", signature = (windows_command=None, unix_command=None))]
    #[pyo3(text_signature = "($self, windows_command, unix_command)")]
    /// (async) Run a shell command.
    ///
    /// Args:
    ///     windows_command (Optional[str]): Command to run on Windows.
    ///     unix_command (Optional[str]): Command to run on Unix.
    ///
    /// Returns:
    ///     CommandOutput: The command output.
    pub fn run_command<'py>(
        &self,
        py: Python<'py>,
        windows_command: Option<String>,
        unix_command: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .run_command(windows_command.as_deref(), unix_command.as_deref())
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = CommandOutput::from(result);
            Ok(py_result)
        })
    }

    #[pyo3(name = "ocr_image_path", text_signature = "($self, image_path)")]
    /// (async) Perform OCR on an image file.
    ///
    /// Args:
    ///     image_path (str): Path to the image file.
    ///
    /// Returns:
    ///     str: The extracted text.
    pub fn ocr_image_path<'py>(
        &self,
        py: Python<'py>,
        image_path: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let image_path = image_path.to_string();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .ocr_image_path(&image_path)
                .await
                .map_err(automation_error_to_pyerr)?;
            Ok(result)
        })
    }

    #[pyo3(name = "ocr_screenshot", text_signature = "($self, screenshot)")]
    /// (async) Perform OCR on a screenshot.
    ///
    /// Args:
    ///     screenshot (ScreenshotResult): The screenshot to process.
    ///
    /// Returns:
    ///     str: The extracted text.
    pub fn ocr_screenshot<'py>(
        &self,
        py: Python<'py>,
        screenshot: &ScreenshotResult,
    ) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let core_screenshot = ::terminator_core::ScreenshotResult {
            image_data: screenshot.image_data.clone(),
            width: screenshot.width,
            height: screenshot.height,
            monitor: screenshot
                .monitor
                .as_ref()
                .map(|m| ::terminator_core::Monitor {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    is_primary: m.is_primary,
                    width: m.width,
                    height: m.height,
                    x: m.x,
                    y: m.y,
                    scale_factor: m.scale_factor,
                }),
        };
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .ocr_screenshot(&core_screenshot)
                .await
                .map_err(automation_error_to_pyerr)?;
            Ok(result)
        })
    }

    #[pyo3(name = "get_current_browser_window")]
    /// (async) Get the currently focused browser window.
    ///
    /// Returns:
    ///     UIElement: The current browser window element.
    pub fn get_current_browser_window<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .get_current_browser_window()
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = UIElement { inner: result };
            Ok(py_result)
        })
    }

    #[pyo3(name = "get_current_window", text_signature = "($self)")]
    /// (async) Get the currently focused window.
    ///
    /// Returns:
    ///     UIElement: The current window element.
    pub fn get_current_window<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .get_current_window()
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = UIElement { inner: result };
            Ok(py_result)
        })
    }

    #[pyo3(name = "get_current_application", text_signature = "($self)")]
    /// (async) Get the currently focused application.
    ///
    /// Returns:
    ///     UIElement: The current application element.
    pub fn get_current_application<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .get_current_application()
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = UIElement { inner: result };
            Ok(py_result)
        })
    }

    #[pyo3(name = "open_url", signature = (url, browser=None))]
    #[pyo3(text_signature = "($self, url, browser)")]
    /// Open a URL in a browser.
    ///
    /// Args:
    ///     url (str): The URL to open.
    ///     browser (Optional[str]): The browser to use. Can be "Default", "Chrome", "Firefox", "Edge", "Brave", "Opera", "Vivaldi", "Arc", or a custom browser path.
    pub fn open_url(&self, url: &str, browser: Option<&str>) -> PyResult<UIElement> {
        let browser_enum = browser.map(|b| match b.to_lowercase().as_str() {
            "default" => ::terminator_core::Browser::Default,
            "chrome" => ::terminator_core::Browser::Chrome,
            "firefox" => ::terminator_core::Browser::Firefox,
            "edge" => ::terminator_core::Browser::Edge,
            "brave" => ::terminator_core::Browser::Brave,
            "opera" => ::terminator_core::Browser::Opera,
            "vivaldi" => ::terminator_core::Browser::Vivaldi,
            "arc" => ::terminator_core::Browser::Arc,
            custom => ::terminator_core::Browser::Custom(custom.to_string()),
        });
        self.inner
            .open_url(url, browser_enum)
            .map(|e| UIElement { inner: e })
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(name = "open_file", text_signature = "($self, file_path)")]
    /// Open a file with its default application.
    ///
    /// Args:
    ///     file_path (str): Path to the file to open.
    pub fn open_file(&self, file_path: &str) -> PyResult<()> {
        self.inner
            .open_file(file_path)
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(
        name = "activate_browser_window_by_title",
        text_signature = "($self, title)"
    )]
    /// Activate a browser window by title.
    ///
    /// Args:
    ///     title (str): The window title to match.
    pub fn activate_browser_window_by_title(&self, title: &str) -> PyResult<()> {
        self.inner
            .activate_browser_window_by_title(title)
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(name = "focused_element", text_signature = "($self)")]
    /// Get the currently focused element.
    ///
    /// Returns:
    ///     UIElement: The focused element.
    pub fn focused_element(&self) -> PyResult<UIElement> {
        self.inner
            .focused_element()
            .map(|e| UIElement { inner: e })
            .map_err(automation_error_to_pyerr)
    }

    #[pyo3(name = "get_window_tree", signature = (pid, title=None, config=None))]
    #[pyo3(text_signature = "($self, pid, title, config)")]
    /// Get the UI tree for a window identified by process ID and optional title.
    ///
    /// Args:
    ///     pid (int): Process ID of the target application.
    ///     title (Optional[str]): Optional window title filter.
    ///     config (Optional[TreeBuildConfig]): Optional configuration for tree building.
    ///
    /// Returns:
    ///     UINode: Complete UI tree starting from the identified window.
    pub fn get_window_tree(
        &self,
        pid: u32,
        title: Option<&str>,
        config: Option<crate::types::TreeBuildConfig>,
    ) -> PyResult<crate::types::UINode> {
        let rust_config = config.map(|c| c.into());
        self.inner
            .get_window_tree(pid, title, rust_config)
            .map(crate::types::UINode::from)
            .map_err(automation_error_to_pyerr)
    }

    // ============== NEW MONITOR METHODS ==============

    #[pyo3(name = "list_monitors", text_signature = "($self)")]
    /// (async) List all available monitors/displays.
    ///
    /// Returns:
    ///     List[Monitor]: List of monitor information.
    pub fn list_monitors<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .list_monitors()
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result: Vec<crate::types::Monitor> = result
                .into_iter()
                .map(crate::types::Monitor::from)
                .collect();
            Ok(py_result)
        })
    }

    #[pyo3(name = "get_primary_monitor", text_signature = "($self)")]
    /// (async) Get the primary monitor.
    ///
    /// Returns:
    ///     Monitor: Primary monitor information.
    pub fn get_primary_monitor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .get_primary_monitor()
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = crate::types::Monitor::from(result);
            Ok(py_result)
        })
    }

    #[pyo3(name = "get_active_monitor", text_signature = "($self)")]
    /// (async) Get the monitor containing the currently focused window.
    ///
    /// Returns:
    ///     Monitor: Active monitor information.
    pub fn get_active_monitor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .get_active_monitor()
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = crate::types::Monitor::from(result);
            Ok(py_result)
        })
    }

    #[pyo3(name = "get_monitor_by_id", text_signature = "($self, id)")]
    /// (async) Get a monitor by its ID.
    ///
    /// Args:
    ///     id (str): The monitor ID to find.
    ///
    /// Returns:
    ///     Monitor: Monitor information.
    pub fn get_monitor_by_id<'py>(&self, py: Python<'py>, id: &str) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let id = id.to_string();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .get_monitor_by_id(&id)
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = crate::types::Monitor::from(result);
            Ok(py_result)
        })
    }

    #[pyo3(name = "get_monitor_by_name", text_signature = "($self, name)")]
    /// (async) Get a monitor by its name.
    ///
    /// Args:
    ///     name (str): The monitor name to find.
    ///
    /// Returns:
    ///     Monitor: Monitor information.
    pub fn get_monitor_by_name<'py>(
        &self,
        py: Python<'py>,
        name: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let name = name.to_string();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .get_monitor_by_name(&name)
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = crate::types::Monitor::from(result);
            Ok(py_result)
        })
    }

    #[pyo3(name = "capture_monitor", text_signature = "($self, monitor)")]
    /// (async) Capture a screenshot of a specific monitor.
    ///
    /// Args:
    ///     monitor (Monitor): The monitor to capture.
    ///
    /// Returns:
    ///     ScreenshotResult: The screenshot data.
    pub fn capture_monitor<'py>(
        &self,
        py: Python<'py>,
        monitor: &crate::types::Monitor,
    ) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let rust_monitor = ::terminator_core::Monitor {
            id: monitor.id.clone(),
            name: monitor.name.clone(),
            is_primary: monitor.is_primary,
            width: monitor.width,
            height: monitor.height,
            x: monitor.x,
            y: monitor.y,
            scale_factor: monitor.scale_factor,
        };
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .capture_monitor(&rust_monitor)
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result = crate::types::ScreenshotResult::from(result);
            Ok(py_result)
        })
    }

    #[pyo3(name = "capture_all_monitors", text_signature = "($self)")]
    /// (async) Capture screenshots of all monitors.
    ///
    /// Returns:
    ///     List[Tuple[Monitor, ScreenshotResult]]: List of monitor and screenshot pairs.
    pub fn capture_all_monitors<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py_with_locals(py, TaskLocals::with_running_loop(py)?, async move {
            let result = desktop
                .capture_all_monitors()
                .await
                .map_err(automation_error_to_pyerr)?;
            let py_result: Vec<(crate::types::Monitor, crate::types::ScreenshotResult)> = result
                .into_iter()
                .map(|(monitor, screenshot)| {
                    (
                        crate::types::Monitor::from(monitor),
                        crate::types::ScreenshotResult::from(screenshot),
                    )
                })
                .collect();
            Ok(py_result)
        })
    }
}
