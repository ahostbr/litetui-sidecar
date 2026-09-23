#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;

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
}

fn initial_view() -> Result<&'static str, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => Ok("timeline"),
        [flag] if flag == "--version" => Ok("version"),
        [flag, view] if flag == "--view" => match view.as_str() {
            "settings" => Ok("settings"),
            "calendar" => Ok("calendar"),
            "job" => Ok("job"),
            _ => Err(format!("invalid sidecar view: {view}")),
        },
        _ => Err("usage: litetui-sidecar [--view settings|calendar|job]".into()),
    }
}
fn main() -> wry::Result<()> {
    let view = match initial_view() {
        Ok("version") => {
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
