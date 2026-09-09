//! Run startup reuses checkpoint storage and portable source validation without an authoring ceremony.

use crate::authoring_error::AuthoringError;
use crate::authoring_protocol::SessionRevision;
use crate::authoring_runtime::AuthoringRuntime;
use crate::cli_composition::parse_cli_interface_source;
use crate::cli_file_read::JsonFileReadGrants;
use crate::cli_interface::{ActiveCliInterface, CliInterfaceOrigin};
use crate::cli_run_routes::CliRunRoutes;
use crate::document_value::MAX_CHECKPOINT_BYTES;
use crate::operation_definition::OperationDefinition;
use crate::session_storage::read_regular_file_bounded;
use crate::storage_faults::StorageFaultControl;
use std::path::{Path, PathBuf};

const RUN_INTENT: &str = "Receiving session created to execute a supplied CLI specification in run mode. This is generated run provenance, not the original authoring task.";

struct TemporaryRunDirectory(PathBuf);
impl Drop for TemporaryRunDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Field order releases the checkpoint lock before removing a temporary run directory.
pub struct CliRunSession {
    pub runtime: AuthoringRuntime,
    pub routes: CliRunRoutes,
    _temporary: Option<TemporaryRunDirectory>,
}

impl CliRunSession {
    pub fn open_cli_run(
        source: &Path,
        checkpoint: Option<&Path>,
        grants: JsonFileReadGrants,
        faults: StorageFaultControl,
    ) -> Result<Self, AuthoringError> {
        let supplied = if source == Path::new("-") {
            None
        } else {
            let bytes =
                read_regular_file_bounded(source, MAX_CHECKPOINT_BYTES, "INVALID_CLI_DEFINITION")?;
            let active = parse_cli_interface_source(
                &bytes,
                None,
                CliInterfaceOrigin {
                    intent_text: RUN_INTENT.into(),
                    revision: SessionRevision(0),
                },
            )?;
            CliRunRoutes::from_cli_definition(&active.definition)?;
            Some(active)
        };
        if supplied.is_none() && checkpoint.is_none_or(|path| !path.exists()) {
            return Err(AuthoringError::new(
                "RUN_SESSION_REQUIRED",
                "Source-free run requires an existing persistent checkpoint.",
                "Supply --session <checkpoint> and use - in place of the source.",
            ));
        }
        let temporary = if checkpoint.is_none() {
            let path = std::env::temp_dir().join(format!("cli-run-{}", uuid::Uuid::new_v4()));
            // The directory owns only this run's files and is inaccessible to other users.
            #[cfg(unix)]
            let created = {
                use std::os::unix::fs::DirBuilderExt;
                std::fs::DirBuilder::new().mode(0o700).create(&path)
            };
            #[cfg(not(unix))]
            let created = std::fs::create_dir(&path);
            created.map_err(|error| {
                AuthoringError::new(
                    "RUN_STORAGE_FAILED",
                    format!("Cannot create temporary run directory: {error}"),
                    "Check local temporary storage and retry.",
                )
            })?;
            Some(TemporaryRunDirectory(path))
        } else {
            None
        };
        let path = checkpoint
            .map(Path::to_path_buf)
            .unwrap_or_else(|| temporary.as_ref().unwrap().0.join("session.json"));
        let declaration =
            OperationDefinition::embedded_composition_definition()?.with_json_read_grants(grants);
        let runtime = if path.exists() {
            let runtime =
                AuthoringRuntime::open_authoring_session(&path, declaration, None, faults)?;
            let active = runtime.session_checkpoint().active_interface.as_ref().ok_or_else(|| AuthoringError::new("NO_ACTIVE_INTERFACE", "Run checkpoint has no active interface.", "Use a run checkpoint or supply a specification with a new checkpoint path."))?;
            if supplied
                .as_ref()
                .is_some_and(|interface| interface.definition_sha256 != active.definition_sha256)
            {
                return Err(AuthoringError::new(
                    "RUN_SPEC_MISMATCH",
                    "Supplied specification differs from the saved run definition.",
                    "Preserve this checkpoint; use its matching specification, - to resume, or a new checkpoint path.",
                ));
            }
            runtime
        } else {
            AuthoringRuntime::create_cli_run_session(
                &path,
                declaration,
                RUN_INTENT.into(),
                supplied.expect("Source required for new run"),
                faults,
            )?
        };
        let routes = CliRunRoutes::from_cli_definition(
            &runtime
                .session_checkpoint()
                .active_interface
                .as_ref()
                .unwrap()
                .definition,
        )?;
        Ok(Self {
            runtime,
            routes,
            _temporary: temporary,
        })
    }

    pub fn active_interface(&self) -> &ActiveCliInterface {
        self.runtime
            .session_checkpoint()
            .active_interface
            .as_ref()
            .expect("Run always has an active interface")
    }
}
