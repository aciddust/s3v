use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{
    NSApp, NSDraggingItem, NSDraggingSource, NSFilePromiseProvider,
    NSFilePromiseProviderDelegate, NSImage, NSView,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString};
use tauri::Manager;

use super::promise::S3FilePromiseDelegate;
use crate::error::AppError;
use crate::state::AppState;

/// Initiate a native macOS drag session for the given S3 keys.
///
/// This is called from the Tauri command on an async task. It captures all
/// required data, then dispatches to the main thread where AppKit drag
/// APIs must be invoked.
pub async fn initiate_drag(
    app: tauri::AppHandle,
    state: &AppState,
    profile_id: String,
    bucket: String,
    keys: Vec<String>,
    common_prefix: String,
) -> Result<(), AppError> {
    let s3_clients = state.s3_clients.clone();
    let profile_manager = state.profile_manager.clone();
    let rt_handle = tokio::runtime::Handle::current();
    let app_for_delegate = app.clone();

    // Obtain the NSView pointer from the first webview window before
    // dispatching to the main thread.
    let webview_window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::S3("No webview window with label 'main' found".into()))?;

    let ns_view_ptr = webview_window
        .ns_view()
        .map_err(|e| AppError::S3(format!("Failed to get NSView: {e}")))?;

    // Store as usize so it's Send. We only dereference on the main thread.
    let ns_view_addr = ns_view_ptr as usize;

    // `run_on_main_thread` requires `FnOnce + Send + 'static`.
    app.run_on_main_thread(move || {
        // SAFETY: run_on_main_thread guarantees we are on the main thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        // Obtain the current event — needed by beginDraggingSession.
        let ns_app = NSApp(mtm);
        let Some(current_event) = ns_app.currentEvent() else {
            eprintln!("[native_drag] No current NSEvent — cannot start drag session");
            return;
        };

        // SAFETY: The address was obtained from Tauri's ns_view() which
        // returns the content view of the window. It remains valid while
        // the window is alive and we are on the main thread.
        let ns_view: &NSView = unsafe { &*(ns_view_addr as *const NSView) };

        // Build one NSDraggingItem per S3 key.
        let mut dragging_items: Vec<Retained<NSDraggingItem>> = Vec::with_capacity(keys.len());
        // We keep delegate references to prevent premature deallocation.
        let mut delegates: Vec<Retained<S3FilePromiseDelegate>> = Vec::with_capacity(keys.len());

        let uti = NSString::from_str("public.data");

        for key in &keys {
            let delegate = S3FilePromiseDelegate::new(
                mtm,
                profile_id.clone(),
                bucket.clone(),
                key.clone(),
                common_prefix.clone(),
                s3_clients.clone(),
                profile_manager.clone(),
                rt_handle.clone(),
                app_for_delegate.clone(),
            );

            // Create promise provider with our delegate.
            let delegate_proto: &ProtocolObject<dyn NSFilePromiseProviderDelegate> =
                ProtocolObject::from_ref(&*delegate);
            let provider =
                NSFilePromiseProvider::initWithFileType_delegate(
                    NSFilePromiseProvider::alloc(),
                    &uti,
                    delegate_proto,
                );

            // Wrap in a NSDraggingItem.
            let pasteboard_writer: &ProtocolObject<dyn objc2_app_kit::NSPasteboardWriting> =
                ProtocolObject::from_ref(&*provider);
            let item = NSDraggingItem::initWithPasteboardWriter(
                NSDraggingItem::alloc(),
                pasteboard_writer,
            );

            // Set a small drag image (a 32x32 transparent icon).
            let size = NSSize::new(32.0, 32.0);
            let drag_image = NSImage::initWithSize(NSImage::alloc(), size);
            let frame = NSRect::new(NSPoint::new(0.0, 0.0), size);
            unsafe {
                item.setDraggingFrame_contents(frame, Some(&*drag_image as &AnyObject));
            }

            delegates.push(delegate);
            dragging_items.push(item);
        }

        if dragging_items.is_empty() {
            return;
        }

        // Use the first delegate as the dragging source (all share the same
        // NSDragOperation::Copy behaviour).
        let source: &ProtocolObject<dyn NSDraggingSource> =
            ProtocolObject::from_ref(&*delegates[0]);

        let ns_array = NSArray::from_retained_slice(&dragging_items);

        let _session = ns_view.beginDraggingSessionWithItems_event_source(
            &ns_array,
            &current_event,
            source,
        );

        // Prevent delegates from being deallocated while the drag session is
        // in progress. macOS retains the promise providers, but the delegates
        // are stored weakly by NSFilePromiseProvider, so we must ensure they
        // outlive the drag.
        //
        // We leak them intentionally — each delegate is a small allocation and
        // only one set is created per drag gesture. If you drag 100 files
        // that's ~100 small objects leaked per session, acceptable for a
        // desktop app.
        for d in delegates {
            std::mem::forget(d);
        }
    })
    .map_err(|e| AppError::S3(format!("Failed to dispatch to main thread: {e}")))?;

    Ok(())
}
