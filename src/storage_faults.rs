//! Explicit test builds can interrupt publication at an observed boundary.

use crate::authoring_error::AuthoringError;
use std::io;

/// Fault control is inert in normal builds and never reads test environment variables there.
#[derive(Clone, Default)]
pub struct StorageFaultControl {
    #[cfg(feature = "fault-injection")]
    plan: Option<std::sync::Arc<std::sync::Mutex<FaultPlan>>>,
}

#[cfg(feature = "fault-injection")]
struct FaultPlan {
    point: String,
    action: String,
    marker: std::path::PathBuf,
    fired: bool,
}

impl StorageFaultControl {
    /// Read explicit fault-test configuration only when the fault-injection feature is enabled.
    pub fn from_test_environment() -> Result<Self, AuthoringError> {
        #[cfg(feature = "fault-injection")]
        if let Ok(point) = std::env::var("CLI_TEST_FAULT_POINT") {
            let invalid = || {
                AuthoringError::new(
                    "INVALID_REQUEST",
                    "Fault injection requires a supported point, action, and marker path.",
                    "Supply all test controls or run a normal build.",
                )
            };
            let action = std::env::var("CLI_TEST_FAULT_ACTION").map_err(|_| invalid())?;
            let marker = std::env::var_os("CLI_TEST_FAULT_MARKER").ok_or_else(invalid)?;
            if !["error", "pause", "exit"].contains(&action.as_str())
                || ![
                    "before_checkpoint_write",
                    "after_checkpoint_write",
                    "after_checkpoint_file_sync",
                    "after_checkpoint_rename",
                    "after_checkpoint_directory_sync",
                    "before_response_delivery",
                    "after_export_publish",
                    "reopen_file_sync",
                    "reopen_directory_sync",
                ]
                .contains(&point.as_str())
            {
                return Err(invalid());
            }
            return Ok(Self {
                plan: Some(std::sync::Arc::new(std::sync::Mutex::new(FaultPlan {
                    point,
                    action,
                    marker: marker.into(),
                    fired: false,
                }))),
            });
        }
        Ok(Self::default())
    }

    /// Record the reached boundary before interrupting so an unlanded fault cannot count as a pass.
    pub fn hit_storage_boundary(&self, point: &str) -> io::Result<()> {
        #[cfg(feature = "fault-injection")]
        if let Some(shared) = &self.plan {
            let mut plan = shared.lock().expect("Fault-test plan mutex");
            if plan.point != point || plan.fired {
                return Ok(());
            }
            use std::io::Write;
            let mut marker = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&plan.marker)?;
            let evidence =
                serde_json::json!({"point":point,"action":plan.action,"pid":std::process::id()});
            marker.write_all(evidence.to_string().as_bytes())?;
            marker.sync_all()?;
            plan.fired = true;
            match plan.action.as_str() {
                "error" => {
                    return Err(io::Error::other(
                        "Injected storage failure after writing its boundary marker",
                    ));
                }
                "exit" => std::process::exit(87),
                "pause" => loop {
                    std::thread::park_timeout(std::time::Duration::from_millis(50));
                },
                _ => unreachable!("Validated fault action"),
            }
        }
        let _ = point;
        Ok(())
    }
}
