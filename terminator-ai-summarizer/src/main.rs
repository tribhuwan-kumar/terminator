use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;
use arboard::Clipboard;
use tracing::{debug, info, error};
use anyhow::{Result, Context};
use crate::utils::{init_logging, Args};
use crate::client::get_mcp_tool_result;
use crate::ollama::summrize_by_ollama;
use global_hotkey::{GlobalHotKeyManager,
    GlobalHotKeyEvent,
    hotkey::{HotKey, Modifiers, Code}
};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod utils;
mod ollama;
mod client;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

    let args = Args::parse();
    info!("initializing summarizer with model: '{}' hotkey: {}", args.model, args.hotkey);

    let hotkey_parts: Vec<&str> = args.hotkey.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in hotkey_parts {
        match part.trim().to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "cmd" | "command" => modifiers |= Modifiers::SUPER,
            key => {
                code = Some(match key {
                    "a" => Code::KeyA,
                    "b" => Code::KeyB,
                    "c" => Code::KeyC,
                    "d" => Code::KeyD,
                    "e" => Code::KeyE,
                    "f" => Code::KeyF,
                    "g" => Code::KeyG,
                    "h" => Code::KeyH,
                    "i" => Code::KeyI,
                    "j" => Code::KeyJ,
                    "k" => Code::KeyK,
                    "l" => Code::KeyL,
                    "m" => Code::KeyM,
                    "n" => Code::KeyN,
                    "o" => Code::KeyO,
                    "p" => Code::KeyP,
                    "q" => Code::KeyQ,
                    "r" => Code::KeyR,
                    "s" => Code::KeyS,
                    "t" => Code::KeyT,
                    "u" => Code::KeyU,
                    "v" => Code::KeyV,
                    "w" => Code::KeyW,
                    "x" => Code::KeyX,
                    "y" => Code::KeyY,
                    "z" => Code::KeyZ,
                    _ => anyhow::bail!("unsupported key: {}", key),
                });
            }
        }
    }

    let hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyJ);

    let model = Arc::new(args.model);
    let system_prompt = Arc::new(args.system_prompt);
    let code = code.context("no key specified in hotkey")?;
    // println!("modifiers: {:?}", modifiers);
    // let hotkey = HotKey::new(Some(modifiers), code);

    let hotkey_manager = GlobalHotKeyManager::new()?;
    hotkey_manager.register(hotkey)?;

    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_visible(false)
        .build(&event_loop)
        .context("failed to create a dummy window")?;

    let hotkey_receiver = GlobalHotKeyEvent::receiver();
    let processing = Arc::new(Mutex::new(false));

    event_loop.run(move |event, _, control_flow| {
        info!("event loop running");
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // check for hotkey events
                info!("event loop");
                if let Ok(event) = hotkey_receiver.try_recv() {
                    if event.id == hotkey.id() {
                        info!("event loop id {:?}", event.id());
                        let model_clone = Arc::clone(&model);
                        let system_prompt_clone = Arc::clone(&system_prompt);
                        let processing_clone = Arc::clone(&processing);
                        tokio::spawn(async move {
                            let mut lock = processing_clone.lock().await;
                            if *lock {
                                info!("already processing a capture request, please wait...");
                                return;
                            }

                            *lock = true;

                            debug!("hotkey detected! capturing current running application...");

                            // get current windows context by mcp tool
                            match get_mcp_tool_result("get_focused_window_tree".to_string(), None).await {
                                Ok(result) => {
                                    debug!("current screen context captured: {}", result);

                                    // Process with Ollama
                                    match summrize_by_ollama(&model_clone, &system_prompt_clone, &result).await {
                                        Ok(response) => {
                                            debug!("ai summarized generated, copying to clipboard...");
                                            match Clipboard::new() {
                                                Ok(mut clipboard) => {
                                                    if let Err(e) = clipboard.set_text(response) {
                                                        error!("failed to copy to clipboard: {}", e);
                                                    } else {
                                                        error!("Context successfully copied to clipboard!");
                                                    }
                                                }
                                                Err(e) => error!("Failed to access clipboard: {}", e),
                                            }
                                        }
                                        Err(e) => error!("Failed to process with Ollama: {}", e),
                                    }
                                }
                                Err(e) => error!("Failed to capture context: {}", e),
                            }
                            *lock = false;
                        });
                    }
                }
            }
            _ => (),
        }
    });
}

