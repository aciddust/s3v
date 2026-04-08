use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, watch, Mutex, RwLock};
use uuid::Uuid;

use super::types::*;
use crate::error::AppError;
use crate::profile::manager::ProfileManager;
use crate::s3::client_pool::{self, ClientPool};

/// Commands sent to the engine worker loop.
enum EngineCommand {
    Pause(String),
    Resume(String),
    Cancel(String),
    JobAdded,
}

/// Internal state for a running/queued transfer job.
struct JobHandle {
    job: TransferJob,
    cancel_tx: Option<watch::Sender<bool>>,
}

pub struct TransferEngine {
    app_handle: tauri::AppHandle,
    s3_clients: ClientPool,
    profile_manager: Arc<ProfileManager>,
    jobs: Arc<RwLock<HashMap<String, Arc<Mutex<JobHandle>>>>>,
    cmd_tx: mpsc::UnboundedSender<EngineCommand>,
    config: TransferConfig,
}

impl TransferEngine {
    pub fn new(
        app_handle: tauri::AppHandle,
        s3_clients: ClientPool,
        profile_manager: Arc<ProfileManager>,
    ) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let config = TransferConfig::default();
        let jobs: Arc<RwLock<HashMap<String, Arc<Mutex<JobHandle>>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let engine = Self {
            app_handle: app_handle.clone(),
            s3_clients: s3_clients.clone(),
            profile_manager: profile_manager.clone(),
            jobs: jobs.clone(),
            cmd_tx,
            config: config.clone(),
        };

        // Spawn worker loop on Tauri's async runtime (available outside tokio context)
        tauri::async_runtime::spawn(worker_loop(
            app_handle,
            s3_clients,
            profile_manager,
            jobs,
            cmd_rx,
            config,
        ));

        engine
    }

    pub async fn enqueue_upload(
        &self,
        profile_id: String,
        local_path: String,
        bucket: String,
        key: String,
    ) -> Result<String, AppError> {
        crate::logger::info(
            "transfer",
            format!("Enqueue upload: {local_path} → {bucket}/{key}"),
        );
        let id = Uuid::new_v4().to_string();
        let job = TransferJob {
            id: id.clone(),
            profile_id,
            transfer_type: TransferType::Upload,
            path: TransferPath {
                local: local_path,
                remote_bucket: bucket,
                remote_key: key,
            },
            status: TransferStatus::Queued,
            progress: TransferProgress {
                bytes_transferred: 0,
                total_bytes: 0,
            },
            error: None,
        };

        let handle = Arc::new(Mutex::new(JobHandle {
            job,
            cancel_tx: None,
        }));

        self.jobs.write().await.insert(id.clone(), handle);
        let _ = self.cmd_tx.send(EngineCommand::JobAdded);
        Ok(id)
    }

    pub async fn enqueue_download(
        &self,
        profile_id: String,
        bucket: String,
        key: String,
        local_path: String,
    ) -> Result<String, AppError> {
        let id = Uuid::new_v4().to_string();
        let job = TransferJob {
            id: id.clone(),
            profile_id,
            transfer_type: TransferType::Download,
            path: TransferPath {
                local: local_path,
                remote_bucket: bucket,
                remote_key: key,
            },
            status: TransferStatus::Queued,
            progress: TransferProgress {
                bytes_transferred: 0,
                total_bytes: 0,
            },
            error: None,
        };

        let handle = Arc::new(Mutex::new(JobHandle {
            job,
            cancel_tx: None,
        }));

        self.jobs.write().await.insert(id.clone(), handle);
        let _ = self.cmd_tx.send(EngineCommand::JobAdded);
        Ok(id)
    }

    pub fn pause(&self, id: String) -> Result<(), AppError> {
        self.cmd_tx
            .send(EngineCommand::Pause(id))
            .map_err(|_| AppError::Transfer("Engine channel closed".into()))
    }

    pub fn resume(&self, id: String) -> Result<(), AppError> {
        self.cmd_tx
            .send(EngineCommand::Resume(id))
            .map_err(|_| AppError::Transfer("Engine channel closed".into()))
    }

    pub fn cancel(&self, id: String) -> Result<(), AppError> {
        self.cmd_tx
            .send(EngineCommand::Cancel(id))
            .map_err(|_| AppError::Transfer("Engine channel closed".into()))
    }

    pub async fn list_transfers(&self) -> Vec<TransferJobSummary> {
        let jobs = self.jobs.read().await;
        let mut summaries = Vec::with_capacity(jobs.len());
        for handle in jobs.values() {
            let h = handle.lock().await;
            summaries.push(TransferJobSummary::from(&h.job));
        }
        summaries
    }
}

fn emit_status(
    app_handle: &tauri::AppHandle,
    id: &str,
    status: &TransferStatus,
    error: Option<String>,
) {
    let _ = app_handle.emit(
        "transfer:status",
        TransferStatusEvent {
            id: id.to_string(),
            status: status.clone(),
            error,
        },
    );
}

fn emit_progress(app_handle: &tauri::AppHandle, id: &str, progress: &TransferProgress) {
    let _ = app_handle.emit(
        "transfer:progress",
        TransferProgressEvent {
            id: id.to_string(),
            bytes_transferred: progress.bytes_transferred,
            total_bytes: progress.total_bytes,
        },
    );
}

fn emit_completed(app_handle: &tauri::AppHandle, job: &TransferJob) {
    let _ = app_handle.emit(
        "transfer:completed",
        TransferCompletedEvent {
            id: job.id.clone(),
            transfer_type: job.transfer_type.clone(),
            path: job.path.clone(),
        },
    );
}

async fn worker_loop(
    app_handle: tauri::AppHandle,
    s3_clients: ClientPool,
    profile_manager: Arc<ProfileManager>,
    jobs: Arc<RwLock<HashMap<String, Arc<Mutex<JobHandle>>>>>,
    mut cmd_rx: mpsc::UnboundedReceiver<EngineCommand>,
    config: TransferConfig,
) {
    loop {
        // Process all pending commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                EngineCommand::Pause(id) => {
                    let jobs_read = jobs.read().await;
                    if let Some(handle) = jobs_read.get(&id) {
                        let mut h = handle.lock().await;
                        if matches!(h.job.status, TransferStatus::Active) {
                            // Signal cancellation to pause (the task will see this and stop)
                            if let Some(tx) = &h.cancel_tx {
                                let _ = tx.send(true);
                            }
                            h.job.status = TransferStatus::Paused;
                            emit_status(&app_handle, &id, &TransferStatus::Paused, None);
                        }
                    }
                }
                EngineCommand::Resume(id) => {
                    let jobs_read = jobs.read().await;
                    if let Some(handle) = jobs_read.get(&id) {
                        let mut h = handle.lock().await;
                        if matches!(h.job.status, TransferStatus::Paused) {
                            h.job.status = TransferStatus::Queued;
                            h.cancel_tx = None;
                            emit_status(&app_handle, &id, &TransferStatus::Queued, None);
                        }
                    }
                }
                EngineCommand::Cancel(id) => {
                    let jobs_read = jobs.read().await;
                    if let Some(handle) = jobs_read.get(&id) {
                        let mut h = handle.lock().await;
                        if matches!(
                            h.job.status,
                            TransferStatus::Queued
                                | TransferStatus::Active
                                | TransferStatus::Paused
                        ) {
                            if let Some(tx) = &h.cancel_tx {
                                let _ = tx.send(true);
                            }
                            h.job.status = TransferStatus::Cancelled;
                            emit_status(&app_handle, &id, &TransferStatus::Cancelled, None);
                        }
                    }
                }
                EngineCommand::JobAdded => {
                    // Just a wake-up signal
                }
            }
        }

        // Count active jobs
        let active_count = {
            let jobs_read = jobs.read().await;
            let mut count = 0usize;
            for handle in jobs_read.values() {
                let h = handle.lock().await;
                if matches!(h.job.status, TransferStatus::Active) {
                    count += 1;
                }
            }
            count
        };

        // Start queued jobs up to max_concurrent
        if active_count < config.max_concurrent {
            let slots = config.max_concurrent - active_count;
            let jobs_read = jobs.read().await;

            // Collect queued job IDs
            let mut queued_ids = Vec::new();
            for (id, handle) in jobs_read.iter() {
                let h = handle.lock().await;
                if matches!(h.job.status, TransferStatus::Queued) {
                    queued_ids.push(id.clone());
                }
                if queued_ids.len() >= slots {
                    break;
                }
            }
            drop(jobs_read);

            for job_id in queued_ids {
                start_job(
                    &app_handle,
                    &s3_clients,
                    &profile_manager,
                    &jobs,
                    &job_id,
                    &config,
                )
                .await;
            }
        }

        // Wait for next command (with timeout to periodically check)
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                // Put the command back by processing it next iteration
                // We can't easily put it back, so we process inline
                match cmd {
                    EngineCommand::Pause(id) => {
                        let jobs_read = jobs.read().await;
                        if let Some(handle) = jobs_read.get(&id) {
                            let mut h = handle.lock().await;
                            if matches!(h.job.status, TransferStatus::Active) {
                                if let Some(tx) = &h.cancel_tx {
                                    let _ = tx.send(true);
                                }
                                h.job.status = TransferStatus::Paused;
                                emit_status(&app_handle, &id, &TransferStatus::Paused, None);
                            }
                        }
                    }
                    EngineCommand::Resume(id) => {
                        let jobs_read = jobs.read().await;
                        if let Some(handle) = jobs_read.get(&id) {
                            let mut h = handle.lock().await;
                            if matches!(h.job.status, TransferStatus::Paused) {
                                h.job.status = TransferStatus::Queued;
                                h.cancel_tx = None;
                                emit_status(&app_handle, &id, &TransferStatus::Queued, None);
                            }
                        }
                    }
                    EngineCommand::Cancel(id) => {
                        let jobs_read = jobs.read().await;
                        if let Some(handle) = jobs_read.get(&id) {
                            let mut h = handle.lock().await;
                            if matches!(h.job.status, TransferStatus::Queued | TransferStatus::Active | TransferStatus::Paused) {
                                if let Some(tx) = &h.cancel_tx {
                                    let _ = tx.send(true);
                                }
                                h.job.status = TransferStatus::Cancelled;
                                emit_status(&app_handle, &id, &TransferStatus::Cancelled, None);
                            }
                        }
                    }
                    EngineCommand::JobAdded => {}
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
        }
    }
}

async fn start_job(
    app_handle: &tauri::AppHandle,
    s3_clients: &ClientPool,
    profile_manager: &Arc<ProfileManager>,
    jobs: &Arc<RwLock<HashMap<String, Arc<Mutex<JobHandle>>>>>,
    job_id: &str,
    config: &TransferConfig,
) {
    let jobs_read = jobs.read().await;
    let Some(handle) = jobs_read.get(job_id) else {
        return;
    };

    let (cancel_tx, cancel_rx) = watch::channel(false);

    let (profile_id, transfer_type, path) = {
        let mut h = handle.lock().await;
        h.job.status = TransferStatus::Active;
        h.cancel_tx = Some(cancel_tx);
        emit_status(app_handle, job_id, &TransferStatus::Active, None);
        (
            h.job.profile_id.clone(),
            h.job.transfer_type.clone(),
            h.job.path.clone(),
        )
    };

    let handle_clone = handle.clone();
    drop(jobs_read);

    let app = app_handle.clone();
    let clients = s3_clients.clone();
    let pm = profile_manager.clone();
    let cfg = config.clone();
    let jid = job_id.to_string();

    tokio::spawn(async move {
        // Get S3 client
        let client_result = async {
            let profile = pm.get(&profile_id).await?;
            client_pool::get_or_create_client(&clients, &profile).await
        }
        .await;

        let client = match client_result {
            Ok(c) => c,
            Err(e) => {
                let mut h = handle_clone.lock().await;
                h.job.status = TransferStatus::Failed;
                h.job.error = Some(e.to_string());
                emit_status(&app, &jid, &TransferStatus::Failed, Some(e.to_string()));
                return;
            }
        };

        let jid_for_progress = jid.clone();
        let app_for_progress = app.clone();
        let handle_for_progress = handle_clone.clone();
        let progress_tx: Arc<dyn Fn(TransferProgress) + Send + Sync> =
            Arc::new(move |progress: TransferProgress| {
                emit_progress(&app_for_progress, &jid_for_progress, &progress);
                // Update job progress (best-effort, don't block)
                let h = handle_for_progress.clone();
                let p = progress.clone();
                tokio::spawn(async move {
                    let mut locked = h.lock().await;
                    locked.job.progress = p;
                });
            });

        let result = match transfer_type {
            TransferType::Upload => {
                super::upload::execute_upload(&client, &path, &cfg, cancel_rx, progress_tx).await
            }
            TransferType::Download => {
                super::download::execute_download(&client, &path, &cfg, cancel_rx, progress_tx)
                    .await
            }
        };

        let mut h = handle_clone.lock().await;
        match result {
            Ok(()) => {
                h.job.status = TransferStatus::Completed;
                emit_status(&app, &jid, &TransferStatus::Completed, None);
                emit_completed(&app, &h.job);
            }
            Err(e) => {
                let msg = e.to_string();
                // If already cancelled/paused, don't overwrite status
                if !matches!(
                    h.job.status,
                    TransferStatus::Cancelled | TransferStatus::Paused
                ) {
                    h.job.status = TransferStatus::Failed;
                    h.job.error = Some(msg.clone());
                    emit_status(&app, &jid, &TransferStatus::Failed, Some(msg));
                }
            }
        }
    });
}
