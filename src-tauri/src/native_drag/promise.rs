use std::sync::Arc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSDragOperation, NSDraggingContext, NSDraggingSession, NSDraggingSource,
    NSFilePromiseProvider, NSFilePromiseProviderDelegate,
};
use objc2_foundation::{NSDictionary, NSError, NSObjectProtocol, NSString, NSURL};

use crate::profile::manager::ProfileManager;
use crate::s3::client_pool::ClientPool;
use crate::transfer::types::{
    TransferCompletedEvent, TransferPath, TransferProgress, TransferProgressEvent,
    TransferStatus, TransferStatusEvent, TransferType,
};
use tauri::Emitter;

/// Ivars stored on each delegate instance.
pub struct DelegateIvars {
    pub profile_id: String,
    pub bucket: String,
    pub key: String,
    pub common_prefix: String,
    pub s3_clients: ClientPool,
    pub profile_manager: Arc<ProfileManager>,
    pub runtime_handle: tokio::runtime::Handle,
    pub app_handle: tauri::AppHandle,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - We do not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "S3FilePromiseDelegate"]
    #[ivars = DelegateIvars]
    pub struct S3FilePromiseDelegate;

    // Mark as conforming to NSObjectProtocol (required by both delegate protocols).
    unsafe impl NSObjectProtocol for S3FilePromiseDelegate {}

    // NSFilePromiseProviderDelegate implementation.
    unsafe impl NSFilePromiseProviderDelegate for S3FilePromiseDelegate {
        #[unsafe(method_id(filePromiseProvider:fileNameForType:))]
        fn file_promise_provider_file_name(
            &self,
            _provider: &NSFilePromiseProvider,
            _file_type: &NSString,
        ) -> Retained<NSString> {
            let key = &self.ivars().key;
            // NSFilePromiseProvider only supports flat filenames (no path separators).
            // Extract just the filename from the key.
            let filename = key
                .rsplit('/')
                .next()
                .filter(|s: &&str| !s.is_empty())
                .unwrap_or(key);
            NSString::from_str(filename)
        }

        #[unsafe(method(filePromiseProvider:writePromiseToURL:completionHandler:))]
        fn file_promise_provider_write_promise(
            &self,
            _provider: &NSFilePromiseProvider,
            url: &NSURL,
            completion_handler: &block2::DynBlock<dyn Fn(*mut NSError)>,
        ) {
            // Clone everything we need before moving into the thread.
            let profile_id = self.ivars().profile_id.clone();
            let bucket = self.ivars().bucket.clone();
            let key = self.ivars().key.clone();
            let s3_clients = self.ivars().s3_clients.clone();
            let profile_manager = self.ivars().profile_manager.clone();
            let rt = self.ivars().runtime_handle.clone();

            // Convert NSURL to a file-system path string.
            let local_path: String = url
                .path()
                .map(|p| p.to_string())
                .unwrap_or_default();

            // Copy the completion handler to the heap so it outlives this
            // scope, then convert to a usize for Send-safe transfer across
            // threads. usize is Send, unlike raw pointers.
            let completion_rc = completion_handler.copy();
            let completion_addr = RcBlock::into_raw(completion_rc) as usize;

            let app_handle = self.ivars().app_handle.clone();
            let profile_id_for_event = profile_id.clone();
            let transfer_id = uuid::Uuid::new_v4().to_string();

            // Emit transfer:added with full job info so the frontend store can track it
            let _ = app_handle.emit(
                "transfer:added",
                crate::transfer::types::TransferJobSummary {
                    id: transfer_id.clone(),
                    profile_id: profile_id_for_event,
                    transfer_type: TransferType::Download,
                    path: TransferPath {
                        local: local_path.clone(),
                        remote_bucket: bucket.clone(),
                        remote_key: key.clone(),
                    },
                    status: TransferStatus::Active,
                    progress: TransferProgress {
                        bytes_transferred: 0,
                        total_bytes: 0,
                    },
                    error: None,
                },
            );

            // Spawn a std::thread so we don't block the Cocoa operation queue
            // or the tokio runtime. Inside, we block_on the async download.
            std::thread::spawn(move || {
                // Reconstruct the RcBlock from the raw pointer address.
                // SAFETY: We have exclusive ownership; the address was obtained
                // from RcBlock::into_raw just above.
                let completion_ptr = completion_addr
                    as *mut block2::Block<dyn Fn(*mut NSError)>;
                let completion = unsafe { RcBlock::from_raw(completion_ptr) }
                    .expect("completion handler pointer was null");

                let result = rt.block_on(async {
                    let profile = profile_manager.get(&profile_id).await?;
                    let client =
                        crate::s3::client_pool::get_or_create_client(&s3_clients, &profile)
                            .await?;

                    // HEAD request to get file size for progress reporting
                    let head = client
                        .head_object()
                        .bucket(&bucket)
                        .key(&key)
                        .send()
                        .await
                        .map_err(|e| crate::error::AppError::Transfer(
                            format!("HEAD request failed: {e}"),
                        ))?;
                    let total_bytes = head.content_length().unwrap_or(0) as u64;

                    // Emit initial progress
                    let _ = app_handle.emit(
                        "transfer:progress",
                        TransferProgressEvent {
                            id: transfer_id.clone(),
                            bytes_transferred: 0,
                            total_bytes,
                        },
                    );

                    // Stream download with progress
                    let resp = client
                        .get_object()
                        .bucket(&bucket)
                        .key(&key)
                        .send()
                        .await
                        .map_err(|e| crate::error::AppError::Transfer(
                            format!("Download failed: {e}"),
                        ))?;

                    if let Some(parent) = std::path::Path::new(&local_path).parent() {
                        tokio::fs::create_dir_all(parent).await.map_err(|e| {
                            crate::error::AppError::Transfer(
                                format!("Failed to create directory: {e}"),
                            )
                        })?;
                    }

                    let mut file = tokio::fs::File::create(&local_path).await.map_err(|e| {
                        crate::error::AppError::Transfer(
                            format!("Failed to create file: {e}"),
                        )
                    })?;

                    use tokio::io::AsyncWriteExt;
                    let mut stream = resp.body;
                    let mut bytes_transferred: u64 = 0;
                    while let Some(chunk) = stream.try_next().await.map_err(|e| {
                        crate::error::AppError::Transfer(
                            format!("Failed to read stream: {e}"),
                        )
                    })? {
                        file.write_all(&chunk).await.map_err(|e| {
                            crate::error::AppError::Transfer(
                                format!("Failed to write chunk: {e}"),
                            )
                        })?;
                        bytes_transferred += chunk.len() as u64;
                        let _ = app_handle.emit(
                            "transfer:progress",
                            TransferProgressEvent {
                                id: transfer_id.clone(),
                                bytes_transferred,
                                total_bytes,
                            },
                        );
                    }

                    file.flush().await.map_err(|e| {
                        crate::error::AppError::Transfer(
                            format!("Failed to flush file: {e}"),
                        )
                    })?;

                    // Emit completed
                    let _ = app_handle.emit(
                        "transfer:status",
                        TransferStatusEvent {
                            id: transfer_id.clone(),
                            status: TransferStatus::Completed,
                            error: None,
                        },
                    );
                    let _ = app_handle.emit(
                        "transfer:completed",
                        TransferCompletedEvent {
                            id: transfer_id.clone(),
                            transfer_type: TransferType::Download,
                            path: TransferPath {
                                local: local_path.clone(),
                                remote_bucket: bucket.clone(),
                                remote_key: key.clone(),
                            },
                        },
                    );

                    Ok::<(), crate::error::AppError>(())
                });

                match result {
                    Ok(()) => {
                        completion.call((std::ptr::null_mut(),));
                    }
                    Err(e) => {
                        eprintln!("[native_drag] S3 download failed: {e}");
                        // Emit failed status
                        let _ = app_handle.emit(
                            "transfer:status",
                            TransferStatusEvent {
                                id: transfer_id.clone(),
                                status: TransferStatus::Failed,
                                error: Some(e.to_string()),
                            },
                        );
                        let domain = NSString::from_str("net.d3fau1t.s3v");
                        let description = NSString::from_str(
                            &format!("S3 download failed: {e}"),
                        );
                        let desc_key = NSError::NSLocalizedDescriptionKey();
                        let user_info: Retained<
                            NSDictionary<NSString, objc2::runtime::AnyObject>,
                        > = unsafe {
                            NSDictionary::dictionaryWithObject_forKey(
                                &description,
                                objc2::runtime::ProtocolObject::from_ref(desc_key),
                            )
                        };
                        let ns_error = unsafe {
                            NSError::initWithDomain_code_userInfo(
                                NSError::alloc(),
                                &domain,
                                1,
                                Some(&user_info),
                            )
                        };
                        completion
                            .call((Retained::as_ptr(&ns_error).cast_mut(),));
                    }
                }
            });
        }
    }

    // NSDraggingSource implementation.
    unsafe impl NSDraggingSource for S3FilePromiseDelegate {
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn dragging_session_source_operation_mask(
            &self,
            _session: &NSDraggingSession,
            _context: NSDraggingContext,
        ) -> NSDragOperation {
            NSDragOperation::Copy
        }
    }
);

impl S3FilePromiseDelegate {
    /// Create a new delegate with the given S3 connection information.
    pub fn new(
        mtm: MainThreadMarker,
        profile_id: String,
        bucket: String,
        key: String,
        common_prefix: String,
        s3_clients: ClientPool,
        profile_manager: Arc<ProfileManager>,
        runtime_handle: tokio::runtime::Handle,
        app_handle: tauri::AppHandle,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(DelegateIvars {
            profile_id,
            bucket,
            key,
            common_prefix,
            s3_clients,
            profile_manager,
            runtime_handle,
            app_handle,
        });
        unsafe { msg_send![super(this), init] }
    }
}
