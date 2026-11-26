use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};

use objc::{
    class, msg_send,
    runtime::{class_addMethod, Class, Object, Sel, BOOL, NO, YES},
    sel, sel_impl,
};
use tauri::{AppHandle, Manager, WindowBuilder};

static DESKTOP: OnceLock<(AppHandle, u16)> = OnceLock::new();
static INSTALLED: AtomicBool = AtomicBool::new(false);

pub fn installed() -> bool {
    INSTALLED.load(Ordering::Relaxed)
}

pub fn install(app: AppHandle, port: u16) {
    if DESKTOP.set((app, port)).is_err() {
        return;
    }

    // Tauri 1 does not expose the macOS reopen event. Add the optional
    // AppKit callback to its existing delegate, retaining all other methods.
    unsafe {
        let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let delegate: *mut Object = msg_send![application, delegate];
        if delegate.is_null() {
            return;
        }
        let class: *mut Class = msg_send![delegate, class];
        let signature = if cfg!(target_arch = "aarch64") {
            b"B@:@B\0"
        } else {
            b"c@:@c\0"
        };
        let added = class_addMethod(
            class,
            sel!(applicationShouldHandleReopen:hasVisibleWindows:),
            std::mem::transmute::<
                extern "C" fn(&Object, Sel, *mut Object, BOOL) -> BOOL,
                objc::runtime::Imp,
            >(reopen),
            signature.as_ptr().cast(),
        );
        INSTALLED.store(added == YES, Ordering::Relaxed);
    }
}

extern "C" fn reopen(_: &Object, _: Sel, _: *mut Object, visible: BOOL) -> BOOL {
    if visible == YES {
        return YES;
    }
    let Some((app, port)) = DESKTOP.get() else {
        return YES;
    };
    if let Some(window) = app.get_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    } else if let Some(config) = app
        .config()
        .tauri
        .windows
        .iter()
        .find(|window| window.label == "main")
    {
        match WindowBuilder::from_config(app, config.clone()).build() {
            Ok(window) => {
                super::apply_default_webview_zoom(&window);
                super::wait_for_backend_then_navigate(window, *port);
            }
            Err(error) => eprintln!("Could not reopen the OneBrain window: {error}"),
        }
    }
    NO
}
