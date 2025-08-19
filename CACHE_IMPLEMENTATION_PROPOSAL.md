# Element Cache System Implementation Proposal

## Problem Statement

The MCP server experiences 4-5 second delays when finding UI elements, particularly with complex selector chains like:

```
role:Pane|name:contains:NETR Online... >> role:hyperlink|name:Florida
```

Debug logs show:

- Tool called: `19:36:17.156`
- Element found: `19:36:21.848` (4.7 seconds delay!)
- Timeout parameter is ignored - searches take full 5+ seconds regardless

## Root Cause Analysis

1. **Windows UI Automation tree traversal is slow** - Takes 5.2 seconds to search the entire UI tree for Pane elements
2. **No caching** - Every search starts from scratch, even for the same element
3. **Timeout not respected** - The Windows API doesn't honor our timeout parameter during tree traversal

## Proposed Solution: Smart Element Cache with Real-time Invalidation

### Architectural Decision: Lightweight Monitoring vs Workflow Recorder

We chose to create **lightweight monitoring in terminator core** rather than reusing `terminator-workflow-recorder` for the following reasons:

#### Why NOT Reuse Workflow Recorder:

- **Heavy dependencies**: Recorder includes rdev, notify, file I/O, MCP conversion - all unnecessary for cache
- **Wrong dependency direction**: Core library shouldn't depend on higher-level recorder
- **Recording overhead**: Includes event tracking, state management we don't need
- **Complex initialization**: COM setup, multiple threads, channels for recording

#### Why Create Lightweight Monitoring:

- **Minimal overhead**: ~200 lines focused solely on detecting UI changes
- **Clean architecture**: Core features belong in core library
- **Always available**: Not conditional on recorder being present
- **Correct dependencies**: Recorder can later use core monitoring
- **Performance focused**: Just signals for cache invalidation, no data collection

The monitoring code pattern is inspired by the workflow recorder's proven approach, but implemented as a minimal, focused solution appropriate for the core library.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    MCP Server                            │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐    ┌──────────────┐   ┌────────────┐ │
│  │   Element    │<───│    Cache     │<──│   Event    │ │
│  │   Finder     │    │   Storage    │   │  Monitor   │ │
│  └──────────────┘    └──────────────┘   └────────────┘ │
│         │                    │                  │        │
│         ▼                    ▼                  ▼        │
│  ┌──────────────────────────────────────────────────┐   │
│  │          Windows UI Automation API               │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Core Components

## 1. Cache Storage (`terminator/src/platforms/windows/element_cache.rs`)

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uiautomation::UIElement;
use tracing::{debug, info, warn};

/// Thread-safe element cache with TTL and smart invalidation
pub struct ElementCache {
    /// Selector string -> (Element, metadata)
    cache: HashMap<String, CachedElement>,
    /// Maximum cache size
    max_size: usize,
    /// Statistics
    stats: CacheStats,
}

struct CachedElement {
    /// The cached UI element reference
    element: Arc<UIElement>,
    /// When this was cached
    timestamp: Instant,
    /// The application this element belongs to
    app_name: Option<String>,
    /// The window title when cached
    window_title: Option<String>,
    /// RuntimeID for validation
    runtime_id: Vec<i32>,
    /// Access count for LRU
    access_count: usize,
    /// Last access time
    last_accessed: Instant,
}

struct CacheStats {
    hits: usize,
    misses: usize,
    invalidations: usize,
    evictions: usize,
}

impl ElementCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            stats: CacheStats::default(),
        }
    }

    /// Get element from cache if valid
    pub fn get(&mut self, key: &str) -> Option<Arc<UIElement>> {
        // Check if exists and not expired
        if let Some(entry) = self.cache.get_mut(key) {
            // Check TTL (30 seconds)
            if entry.timestamp.elapsed() < Duration::from_secs(30) {
                // Validate element is still valid
                if Self::is_element_valid(&entry.element) {
                    entry.access_count += 1;
                    entry.last_accessed = Instant::now();
                    self.stats.hits += 1;
                    debug!("Cache HIT for selector: {}", key);
                    return Some(Arc::clone(&entry.element));
                }
            }
            // Element expired or invalid
            debug!("Cache entry expired/invalid for: {}", key);
            self.cache.remove(key);
            self.stats.invalidations += 1;
        }

        self.stats.misses += 1;
        debug!("Cache MISS for selector: {}", key);
        None
    }

    /// Insert element into cache
    pub fn insert(&mut self, key: String, element: Arc<UIElement>) {
        // LRU eviction if at capacity
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        let runtime_id = element.get_runtime_id().unwrap_or_default();
        let app_name = Self::get_app_name(&element);
        let window_title = Self::get_window_title(&element);

        self.cache.insert(key.clone(), CachedElement {
            element,
            timestamp: Instant::now(),
            app_name,
            window_title,
            runtime_id,
            access_count: 0,
            last_accessed: Instant::now(),
        });

        info!("Cached element for selector: {}", key);
    }

    /// Invalidate cache entries for a specific application
    pub fn invalidate_app(&mut self, app_name: &str) {
        let keys_to_remove: Vec<_> = self.cache
            .iter()
            .filter(|(_, v)| v.app_name.as_deref() == Some(app_name))
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
            self.stats.invalidations += 1;
        }

        if !keys_to_remove.is_empty() {
            info!("Invalidated {} cache entries for app: {}", keys_to_remove.len(), app_name);
        }
    }

    /// Clear all cache entries except for the specified app
    pub fn invalidate_except(&mut self, keep_app: Option<String>) {
        let keys_to_remove: Vec<_> = self.cache
            .iter()
            .filter(|(_, v)| v.app_name != keep_app)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
            self.stats.invalidations += 1;
        }

        info!("Invalidated {} cache entries (kept: {:?})", keys_to_remove.len(), keep_app);
    }

    /// Check if element is still valid
    fn is_element_valid(element: &UIElement) -> bool {
        // Try to get a property to check if element is still alive
        element.get_name().is_ok()
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some((key, _)) = self.cache
            .iter()
            .min_by_key(|(_, v)| v.last_accessed)
            .map(|(k, v)| (k.clone(), v.last_accessed))
        {
            self.cache.remove(&key);
            self.stats.evictions += 1;
            debug!("Evicted LRU cache entry: {}", key);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
}

// Global cache instance
lazy_static::lazy_static! {
    pub static ref ELEMENT_CACHE: Arc<Mutex<ElementCache>> =
        Arc::new(Mutex::new(ElementCache::new(100)));
}
```

## 2. Lightweight Event Monitor (`terminator/src/platforms/windows/event_monitor.rs`)

Create lightweight monitoring in terminator core (NOT reusing workflow recorder to avoid heavy dependencies):

```rust
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use uiautomation::{UIAutomation, events::*};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use tracing::{info, debug, error};

/// Lightweight event monitor for cache invalidation
/// This is a minimal implementation focused solely on detecting UI changes,
/// without the recording overhead of terminator-workflow-recorder
pub struct EventMonitor {
    automation: UIAutomation,
    handlers: Arc<Mutex<Vec<Box<dyn EventHandler>>>>,
    running: Arc<AtomicBool>,
}

/// Trait for handling UI events
pub trait EventHandler: Send + Sync {
    /// Called when focus changes to a different element
    fn on_focus_change(&self, element: &UIElement);

    /// Called when switching between applications
    fn on_application_switch(&self, old_app: Option<&str>, new_app: &str);

    /// Called when window title changes (useful for browser navigation)
    fn on_window_change(&self, app: &str, old_title: Option<&str>, new_title: &str);
}

/// Cache-specific event handler
pub struct CacheInvalidator {
    cache: Arc<Mutex<ElementCache>>,
}

impl EventHandler for CacheInvalidator {
    fn on_focus_change(&self, element: &UIElement) {
        // Light invalidation - just mark potential stale entries
        if let Ok(mut cache) = self.cache.lock() {
            cache.mark_potentially_stale();
        }
    }

    fn on_application_switch(&self, old_app: Option<&str>, new_app: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            // Clear cache for old application
            if let Some(app) = old_app {
                debug!("App switch: {} -> {}, invalidating old app cache", app, new_app);
                cache.invalidate_app(app);
            }
        }
    }

    fn on_window_change(&self, app: &str, _old_title: Option<&str>, new_title: &str) {
        // Detect browser navigation
        if app.contains("Chrome") || app.contains("Firefox") || app.contains("Edge") {
            if new_title.contains("http") || new_title != _old_title.unwrap_or("") {
                if let Ok(mut cache) = self.cache.lock() {
                    debug!("Browser navigation detected, invalidating browser elements");
                    cache.invalidate_browser_elements();
                }
            }
        }
    }
}

impl EventMonitor {
    pub fn new() -> Result<Self> {
        // Initialize COM for UI Automation (single-threaded apartment)
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
        }

        let automation = UIAutomation::new()?;

        Ok(Self {
            automation,
            handlers: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Add an event handler
    pub fn add_handler(&self, handler: Box<dyn EventHandler>) {
        if let Ok(mut handlers) = self.handlers.lock() {
            handlers.push(handler);
        }
    }

    /// Start lightweight monitoring (minimal overhead)
    pub fn start(&self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(()); // Already running
        }

        info!("Starting lightweight event monitor");

        // Simple focus change detection
        let handlers = Arc::clone(&self.handlers);
        let (tx, rx) = std::sync::mpsc::channel::<FocusEvent>();

        struct LightweightFocusHandler {
            sender: std::sync::mpsc::Sender<FocusEvent>,
        }

        impl CustomFocusChangedEventHandler for LightweightFocusHandler {
            fn handle(&self, element: &UIElement) -> uiautomation::Result<()> {
                // Minimal processing - just extract app/window info
                let event = FocusEvent {
                    app_name: element.application_name().ok(),
                    window_title: element.window_title().ok(),
                };
                self.sender.send(event).ok();
                Ok(())
            }
        }

        let handler = UIFocusChangedEventHandler::from(LightweightFocusHandler { sender: tx });
        self.automation.add_focus_changed_event_handler(None, &handler)?;

        // Lightweight processing thread
        let running = Arc::clone(&self.running);
        std::thread::spawn(move || {
            let mut last_app = None;
            let mut last_title = None;

            while let Ok(event) = rx.recv() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                // Notify handlers of changes
                if let Ok(handlers) = handlers.lock() {
                    // Application switch detection
                    if last_app != event.app_name {
                        if let Some(new_app) = &event.app_name {
                            for handler in handlers.iter() {
                                handler.on_application_switch(last_app.as_deref(), new_app);
                            }
                        }
                        last_app = event.app_name.clone();
                    }

                    // Window title change detection
                    if last_title != event.window_title {
                        if let (Some(app), Some(title)) = (&event.app_name, &event.window_title) {
                            for handler in handlers.iter() {
                                handler.on_window_change(app, last_title.as_deref(), title);
                            }
                        }
                        last_title = event.window_title.clone();
                    }
                }
            }
        });

        self.running.store(true, Ordering::SeqCst);
        info!("Lightweight event monitor started successfully");
        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Event monitor stopped");
    }
}

struct FocusEvent {
    app_name: Option<String>,
    window_title: Option<String>,
}
```

## 3. Integration with Windows Engine (`terminator/src/platforms/windows/engine.rs`)

Modify the existing engine to use cache:

```rust
// Add to WindowsEngine struct (line ~184)
pub struct WindowsEngine {
    pub automation: ThreadSafeWinUIAutomation,
    use_background_apps: bool,
    activate_app: bool,
    // NEW: Element cache
    cache: Arc<Mutex<ElementCache>>,
    // NEW: Event monitor for cache invalidation
    event_monitor: Option<EventMonitor>,
}

// Modify find_elements method (line ~430)
fn find_elements(
    &self,
    selector: &Selector,
    root: Option<&UIElement>,
    timeout: Option<Duration>,
    depth: Option<usize>,
) -> Result<Vec<UIElement>, AutomationError> {
    // Generate cache key
    let cache_key = Self::generate_cache_key(selector, root);

    // 1. CHECK CACHE FIRST
    {
        let mut cache = self.cache.lock().unwrap();
        if let Some(cached_element) = cache.get(&cache_key) {
            info!("🎯 Cache HIT! Avoided {}ms search", timeout.as_millis());
            return Ok(vec![UIElement::from(cached_element)]);
        }
    }

    // 2. CACHE MISS - Do the expensive search
    let start = Instant::now();

    // ... existing search logic ...
    let elements = self.find_elements_internal(selector, root, timeout, depth)?;

    let search_time = start.elapsed();
    info!("Search took {}ms for selector: {:?}", search_time.as_millis(), selector);

    // 3. CACHE THE RESULT (only for expensive searches)
    if search_time > Duration::from_millis(500) && !elements.is_empty() {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(cache_key, Arc::new(elements[0].clone()));
    }

    Ok(elements)
}

// Helper to generate cache key
fn generate_cache_key(selector: &Selector, root: Option<&UIElement>) -> String {
    let root_key = root.map(|r| format!("{}:{}", r.role(), r.name().unwrap_or_default()))
        .unwrap_or_else(|| "desktop".to_string());
    format!("{}@{}", selector.to_string(), root_key)
}
```

## 4. MCP Server Integration (`terminator-mcp-agent/src/server.rs`)

Enable event monitor in MCP server:

```rust
// In DesktopWrapper initialization
impl DesktopWrapper {
    pub fn new() -> Self {
        let desktop = Desktop::new_with_cache(true); // Enable caching

        // Start lightweight event monitor for intelligent cache invalidation
        if let Some(monitor) = desktop.get_event_monitor() {
            // Add cache invalidator handler
            let invalidator = Box::new(CacheInvalidator::new(desktop.get_cache()));
            monitor.add_handler(invalidator);
            monitor.start().ok();
        }

        Self {
            desktop: Arc::new(desktop),
            // ... other fields
        }
    }
}

// Add cache stats endpoint
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetCacheStatsArgs {}

async fn get_cache_stats(&self, _args: GetCacheStatsArgs) -> Result<Value> {
    let stats = self.desktop.get_cache_stats();
    Ok(json!({
        "hits": stats.hits,
        "misses": stats.misses,
        "hit_rate": stats.hit_rate(),
        "size": stats.size,
        "invalidations": stats.invalidations,
        "evictions": stats.evictions,
    }))
}
```

## 5. Tree Building Cache (`terminator/src/platforms/windows/tree_builder.rs`)

Cache frequently accessed subtrees during `get_window_tree`:

```rust
// Modify build_ui_node_tree_configurable (line ~500)
pub fn build_ui_node_tree_configurable(
    element: &UIElement,
    config: &TreeBuildingConfig,
    context: &mut TreeBuildingContext,
) -> Result<UINode> {
    // Check if this subtree is cached
    let subtree_key = format!("tree:{}:{}", element.runtime_id()?, element.name()?);

    if let Some(cached_tree) = context.tree_cache.get(&subtree_key) {
        if cached_tree.age < Duration::from_secs(5) {
            return Ok(cached_tree.clone());
        }
    }

    // Build tree (existing logic)
    let node = build_tree_internal(element, config, context)?;

    // Cache significant subtrees (Panes, Windows)
    if element.role() == "Pane" || element.role() == "Window" {
        context.tree_cache.insert(subtree_key, node.clone());
    }

    // POPULATE ELEMENT CACHE during traversal
    if should_cache_element(&element) {
        let selector = generate_selector_for_element(&element);
        ELEMENT_CACHE.lock().unwrap().insert(selector, Arc::new(element.clone()));
    }

    Ok(node)
}

fn should_cache_element(element: &UIElement) -> bool {
    let role = element.role();
    let name = element.name().unwrap_or_default();

    // Cache significant named containers
    (role == "Pane" || role == "Window") && name.len() > 50
}
```

## Implementation Timeline

### Phase 1: Basic Cache (2-3 hours)

1. Create `element_cache.rs` with basic get/set operations
2. Integrate into `WindowsEngine::find_elements`
3. Add cache stats logging

### Phase 2: Smart Invalidation (3-4 hours)

1. Create lightweight `event_monitor.rs` in terminator core
2. Implement minimal focus change detection
3. Add EventHandler trait and CacheInvalidator
4. Set up app-switch and navigation invalidation

### Phase 3: Optimization (2-3 hours)

1. Add tree-building cache
2. Implement LRU eviction
3. Fine-tune TTL values
4. Add cache warming strategies

### Phase 4: Testing & Metrics (2 hours)

1. Add cache stats endpoint
2. Create benchmark tests
3. Measure performance improvements
4. Document configuration options

## Expected Performance Improvements

### Before (Current State)

- First search: 5.2 seconds
- Repeat search: 5.2 seconds
- Workflow with 10 searches: 52 seconds

### After (With Cache)

- First search: 5.2 seconds (cache miss)
- Repeat search: <10ms (cache hit)
- Workflow with 10 searches: ~5.5 seconds (90% cache hits)

### Performance Gains

- **10-100x faster** for cached elements
- **~90% reduction** in workflow execution time
- **Immediate** element access for repeated operations

## Risk Mitigation

### Stale References

- **Solution**: Validate element on cache hit using `get_name()`
- **Fallback**: Remove from cache and re-search if invalid

### Memory Usage

- **Solution**: LRU eviction at 100 elements max
- **Monitoring**: Track cache size and eviction rate

### Cache Coherency

- **Solution**: Real-time invalidation via focus monitoring
- **TTL**: 30-second expiry as safety net

## Configuration Options

```toml
[cache]
enabled = true
max_size = 100
ttl_seconds = 30
monitor_enabled = true
warm_on_startup = false
log_stats_interval = 60
```

## Debugging Tools

### Cache Stats Logging

```
[INFO] Cache Stats: hits=450 misses=50 hit_rate=90% size=75/100
[DEBUG] Cache HIT for selector: role:Pane|name:NETR Online
[DEBUG] Cache invalidated 12 entries for app switch: Chrome -> Notepad
```

### MCP Tool for Cache Management

```json
{
  "tool": "manage_cache",
  "arguments": {
    "action": "stats" | "clear" | "invalidate_app",
    "app_name": "Chrome"
  }
}
```

## Success Metrics

1. **Primary**: Reduce element finding time from 5s to <100ms for cached elements
2. **Hit Rate**: Achieve >80% cache hit rate in typical workflows
3. **Reliability**: Zero incorrect element returns due to stale cache
4. **Memory**: Keep cache memory usage under 10MB

## References

- Existing Process Cache Pattern: `terminator/src/platforms/windows/applications.rs:658-765`
- Monitoring Pattern Inspiration: `terminator-workflow-recorder/src/recorder/windows/mod.rs:1030-1179` (pattern only, not reusing code)
- Windows Property Cache: `terminator/src/platforms/windows/element.rs:151-167`
- UI Automation Events: `uiautomation` crate event handling

## Next Steps

1. Review and approve this proposal
2. Create feature branch: `feature/element-cache`
3. Implement Phase 1 (basic cache)
4. Test and measure improvements
5. Iterate based on results
