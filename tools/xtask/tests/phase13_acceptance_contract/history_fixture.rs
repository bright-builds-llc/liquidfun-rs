//! Synthetic test-only provenance; never reads or rewrites real promotion evidence.

use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

use super::closure::{
    changed_path_set_sha256, derive_replay_closure, derive_witness_closure,
    promoted_path_set_sha256, receipt_semantic_sha256, reviewed_content_digests_from_root,
};
use super::{
    ARTIFACT_MANIFEST_PATH, Command, EXACT_BYTES_DIGEST_MODE, MATERIALS_MANIFEST, ORACLE_REVISION,
    PROMOTED_PATHS, Path, PathBuf, RECEIPT_PATH, RECEIPT_SEMANTIC_DIGEST_MODE,
    REPLAY_EVIDENCE_PATH, SOURCE_MAP_PATH, WITNESS_PROVENANCE_PATH, fs, sha256,
};

type FixtureResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
const BUNDLE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CATALOG_PATH: &str = "crates/liquidfun-differential/src/fixtures/replay/catalog.rs";
const WITNESS_PATH: &str = "reference/artifacts/phase9/lifecycle-contact-witnesses.json";

pub(crate) struct HistoryFixture {
    root: PathBuf,
}

impl HistoryFixture {
    pub(crate) fn new() -> FixtureResult<Self> {
        let root = std::env::temp_dir().join(format!(
            "liquidfun-acceptance-history-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&root)?;
        let fixture = Self { root };
        fixture.git(&["init", "-q", "-b", "main"])?;
        fixture.git(&["config", "user.name", "Synthetic acceptance fixture"])?;
        fixture.git(&["config", "user.email", "fixture@example.invalid"])?;
        fixture.git(&["config", "commit.gpgsign", "false"])?;
        fixture.git(&["config", "core.hooksPath", ".git/hooks"])?;
        fixture.seed_baseline()?;
        let producer = fixture.head()?;
        fixture.commit_change("notes.txt", b"promotion base metadata\n")?;
        let base = fixture.head()?;
        fixture.promote(&producer, &base)?;
        Ok(fixture)
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn head(&self) -> FixtureResult<String> {
        self.git(&["rev-parse", "HEAD"])
    }

    pub(crate) fn commit_change(&self, name: &str, bytes: &[u8]) -> FixtureResult {
        self.write(name, bytes)?;
        self.git(&["add", "--", name])?;
        self.git(&["commit", "-q", "-m", "synthetic descendant"])?;
        Ok(())
    }

    pub(crate) fn replace_receipt_schema(&self, schema: u32) -> FixtureResult {
        let mut receipt: Value = serde_json::from_slice(&fs::read(self.root.join(RECEIPT_PATH))?)?;
        receipt["schema_version"] = json!(schema);
        self.write_json(RECEIPT_PATH, &receipt)
    }

    fn seed_baseline(&self) -> FixtureResult {
        self.write_json(
            MATERIALS_MANIFEST,
            &json!({
                "schema_version": 1, "target": "phase9-lifecycle-contact-witness",
                "preset": "oracle-debug", "materials": [],
            }),
        )?;
        self.write("Cargo.toml", b"synthetic replay input\n")?;
        for name in PROMOTED_PATHS {
            self.write(name, b"unpromoted fixture bytes\n")?;
            self.git(&["add", "--", name])?;
        }
        self.git(&["add", "--", MATERIALS_MANIFEST, "Cargo.toml"])?;
        self.git(&["commit", "-q", "-m", "synthetic producer"])?;
        Ok(())
    }

    fn promote(&self, producer: &str, base: &str) -> FixtureResult {
        let paths = PROMOTED_PATHS.map(str::to_owned).to_vec();
        let mut changed = paths.clone();
        changed.sort();
        let mut receipt = json!({
            "schema_version": 2, "producer_sha": producer, "bundle_sha256": BUNDLE,
            "promotion_base_sha": base,
            "acquisition": {
                "repository": "fixture/repository", "run_id": 1, "artifact_id": 2,
                "artifact_name": "synthetic", "provider_digest": format!("sha256:{BUNDLE}"),
                "artifact_created_at": "2026-01-01T00:00:00Z",
                "artifact_expires_at": "2026-01-02T00:00:00Z",
            },
            "independent_reviewer_id": "synthetic-test-reviewer",
            "promoted_paths": paths, "promoted_path_set_sha256": promoted_path_set_sha256(&paths)?,
            "promoted_content_sha256": "", "changed_paths": changed, "unchanged_paths": [],
            "changed_path_set_sha256": changed_path_set_sha256(&changed)?, "changed_content_sha256": "",
            "producer_closures": {
                "witness_sha256": derive_witness_closure(&self.root, base)?,
                "replay_sha256": derive_replay_closure(&self.root, base, base)?,
                "recomputed_at_r": true,
            },
            "q_contract": {
                "required_first_parent": base,
                "required_trailers": {
                    "Phase13-Bundle-SHA256": BUNDLE, "Phase13-Producer-SHA": producer,
                    "Phase13-Promotion-Base-SHA": base,
                },
                "q_sha_recorded": false, "acceptance_sha_recorded": false,
            },
        });
        self.write_json(RECEIPT_PATH, &receipt)?;
        self.write_evidence(producer)?;
        self.write_ledgers(producer)?;
        let (promoted, changed_digest) = reviewed_content_digests_from_root(&self.root, &changed)?;
        receipt["promoted_content_sha256"] = json!(promoted);
        receipt["changed_content_sha256"] = json!(changed_digest);
        self.write_json(RECEIPT_PATH, &receipt)?;
        for name in PROMOTED_PATHS {
            self.git(&["add", "--", name])?;
        }
        self.git(&["commit", "-q", "-m", &format!(
            "synthetic promotion\n\nPhase13-Bundle-SHA256: {BUNDLE}\nPhase13-Producer-SHA: {producer}\nPhase13-Promotion-Base-SHA: {base}"
        )])?;
        Ok(())
    }

    fn write_evidence(&self, producer: &str) -> FixtureResult {
        self.write(CATALOG_PATH, b"promoted synthetic catalog\n")?;
        self.write(WITNESS_PATH, b"synthetic witness\n")?;
        self.write_json(
            WITNESS_PROVENANCE_PATH,
            &json!({
                "repository_revision": producer, "oracle_revision": ORACLE_REVISION,
                "compiler_id": "Clang", "compiler_version": "22.1.8",
                "target": "x86_64-unknown-linux-gnu", "cmake_preset": "oracle-debug",
            }),
        )?;
        self.write_json(
            REPLAY_EVIDENCE_PATH,
            &json!({
                "upstream_revision": ORACLE_REVISION, "d1_oracle_identity_sha256": BUNDLE,
            }),
        )
    }

    fn write_ledgers(&self, producer: &str) -> FixtureResult {
        let mut manifest = String::new();
        let mut source_map = String::new();
        for name in [
            WITNESS_PATH,
            WITNESS_PROVENANCE_PATH,
            REPLAY_EVIDENCE_PATH,
            RECEIPT_PATH,
        ] {
            let bytes = fs::read(self.root.join(name))?;
            let (digest, mode) = if name == RECEIPT_PATH {
                (
                    receipt_semantic_sha256(&bytes)?,
                    RECEIPT_SEMANTIC_DIGEST_MODE,
                )
            } else {
                (sha256(&bytes), EXACT_BYTES_DIGEST_MODE)
            };
            write!(
                manifest,
                "[[artifact_schemas.phase13_evidence.records]]\npath = {name:?}\nsha256 = {digest:?}\ndigest_mode = {mode:?}\nproducer_sha = {producer:?}\nbundle_sha256 = {BUNDLE:?}\n"
            )?;
            write!(source_map, "[[mapping]]\nlocal_path = {name:?}\n")?;
        }
        self.write(ARTIFACT_MANIFEST_PATH, manifest.as_bytes())?;
        self.write(SOURCE_MAP_PATH, source_map.as_bytes())
    }

    fn write(&self, name: &str, bytes: &[u8]) -> FixtureResult {
        let target = self.root.join(name);
        fs::create_dir_all(target.parent().ok_or("fixture file has no parent")?)?;
        fs::write(target, bytes)?;
        Ok(())
    }

    fn write_json(&self, name: &str, value: &Value) -> FixtureResult {
        self.write(name, &serde_json::to_vec(value)?)
    }

    fn git(&self, args: &[&str]) -> FixtureResult<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
        }
        Ok(String::from_utf8(output.stdout)?.trim().to_owned())
    }
}

impl Drop for HistoryFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("owned disposable fixture should be removable");
    }
}
