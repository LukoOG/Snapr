use crate::{
    error::SnaprResult,
    models::{Snapshot, reports},
    verification,
};

pub fn handle_verify(snapshots: &[Snapshot]) -> SnaprResult<reports::VerifyReport> {
    verification::verifier::verify_repository(snapshots)
}
