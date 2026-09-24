#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod parent_transport;

use litetui_sidecar::parent_protocol::ParentFrame;
use std::{cell::Cell, rc::Rc};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, Theme, WindowBuilder},
};
use wry::{PageLoadEvent, WebViewBuilder};

const WINDOW_ICON_RGBA: &[u8] = include_bytes!("../assets/windows/sidecar-window.rgba");
const WINDOW_ICON_SIZE: u32 = 64;

enum UserEvent {
    PageReady,
    Parent(parent_transport::ParentEvent),
}

fn initial_view() -> Result<(&'static str, Option<String>), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => Ok(("timeline", None)),
        [flag] if flag == "--version" => Ok(("version", None)),
        [flag, view] if flag == "--view" => match view.as_str() {
            "timeline" => Ok(("timeline", None)),
            "settings" => Ok(("settings", None)),
            "calendar" => Ok(("calendar", None)),
            "job" => Ok(("job", None)),
            _ => Err(format!("invalid sidecar view: {view}")),
        },
        [flag, view, pipe] if flag == "--view" && pipe == "--parent-pipe" => {
            let capability = std::env::var("LITETUI_SIDECAR_TOKEN")
                .map_err(|_| "missing parent capability".to_string())?;
            if capability.len() < 32 {
                return Err("invalid parent capability".into());
            }
            match view.as_str() {
                "timeline" => Ok(("timeline", Some(capability))),
                "settings" => Ok(("settings", Some(capability))),
                "calendar" => Ok(("calendar", Some(capability))),
                "job" => Ok(("job", Some(capability))),
                _ => Err(format!("invalid sidecar view: {view}")),
            }
        }
        _ => Err(
            "usage: litetui-sidecar [--view timeline|settings|calendar|job [--parent-pipe]]".into(),
        ),
    }
}
fn main() -> wry::Result<()> {
    let view = match initial_view() {
        Ok(("version", _)) => {
            println!("litetui-sidecar {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Ok(view) => view,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let (view, capability) = view;
    if let Some(token) = capability {
        parent_transport::start(event_loop.create_proxy(), token);
    }
    let proxy = event_loop.create_proxy();
    let icon = Icon::from_rgba(
        WINDOW_ICON_RGBA.to_vec(),
        WINDOW_ICON_SIZE,
        WINDOW_ICON_SIZE,
    )
    .expect("embedded window icon");
    let window = WindowBuilder::new()
        .with_title("LiteTUI · Sidecar (preview)")
        .with_window_icon(Some(icon))
        .with_inner_size(LogicalSize::new(1440.0, 900.0))
        .with_min_inner_size(LogicalSize::new(900.0, 650.0))
        .with_theme(Some(Theme::Dark))
        .with_visible(false)
        .build(&event_loop)
        .expect("sidecar window");
    let visible = Rc::new(Cell::new(false));
    let webview = WebViewBuilder::new()
        .with_background_color((10, 10, 11, 255))
        .with_custom_protocol("sidecar".into(), |_webview_id, request| {
            assets::response(request)
        })
        .with_url(format!("sidecar://localhost/index.html#{}", view))
        .with_navigation_handler(|url| {
            url.starts_with("sidecar://localhost/") || url.starts_with("http://sidecar.localhost/")
        })
        .with_on_page_load_handler(move |event, _url| {
            if matches!(event, PageLoadEvent::Finished) {
                let _ = proxy.send_event(UserEvent::PageReady);
            }
        })
        .build(&window)?;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(UserEvent::PageReady) if !visible.replace(true) => {
                window.set_visible(true);
                window.set_focus();
            }
            Event::UserEvent(UserEvent::Parent(parent_transport::ParentEvent::Frame(frame))) => {
                handle_parent(frame, &webview);
            }
            Event::UserEvent(UserEvent::Parent(parent_transport::ParentEvent::Disconnected)) => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
        // Keep the trusted offline view alive for the lifetime of its native window.
        let _ = &webview;
    });
}

fn handle_parent(frame: ParentFrame, webview: &wry::WebView) {
    let result = match frame.command.as_str() {
        "hello" => {
            serde_json::json!({"version": litetui_sidecar::parent_protocol::VERSION, "ready": true})
        }
        "open" => {
            let view = frame
                .payload
                .get("view")
                .and_then(serde_json::Value::as_str);
            match view {
                Some("settings" | "calendar" | "job" | "timeline") => {
                    let script = format!("window.SidecarShell?.open({});", serde_json::json!(view));
                    let _ = webview.evaluate_script(&script);
                    serde_json::json!({"opened": view})
                }
                _ => serde_json::json!({"error": "invalid view"}),
            }
        }
        "shutdown" => {
            // A bounded parent termination owns the child after this acknowledgement.
            serde_json::json!({"closing": true})
        }
        _ => serde_json::json!({"error": "unsupported command"}),
    };
    let _ = parent_transport::reply(&frame, result);
}
