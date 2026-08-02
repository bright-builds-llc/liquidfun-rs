use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{TestResult, workspace_root};
use serde_json::{Value, json};

#[path = "producer/lifecycle.rs"]
mod lifecycle;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct ProducerFixture {
    root: PathBuf,
    repository: PathBuf,
    manifest: PathBuf,
    fake_bin: PathBuf,
    candidate: String,
    gh_journal: PathBuf,
    gh_state: PathBuf,
    command_journal: PathBuf,
}

fn run_git(repository: &Path, arguments: &[&str]) -> TestResult<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repository)
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn write_executable(path: &Path, source: &str) -> TestResult {
    fs::write(path, source)?;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

impl ProducerFixture {
    // The constructor deliberately keeps the fake repository, command manifest,
    // and fake GitHub boundary together so lifecycle tests share one exact setup.
    #[allow(clippy::too_many_lines)]
    fn new(include_workflow: bool, helper_mode: &str) -> TestResult<Self> {
        let root = env::temp_dir().join(format!(
            "liquidfun-phase13-1-producer-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            return Err("producer fixture root already exists".into());
        }
        let repository = root.join("repository");
        let remote = root.join("remote.git");
        let fake_bin = root.join("bin");
        let command_journal = root.join("command-journal.txt");
        fs::create_dir_all(&repository)?;
        fs::create_dir_all(&fake_bin)?;
        run_git(&repository, &["init", "-q", "-b", "main"])?;
        run_git(
            &repository,
            &["config", "user.email", "fixture@example.invalid"],
        )?;
        run_git(&repository, &["config", "user.name", "Fixture"])?;
        fs::write(repository.join("source.txt"), "base\n")?;
        run_git(&repository, &["add", "source.txt"])?;
        run_git(&repository, &["commit", "-q", "-m", "base"])?;
        let parent = run_git(&repository, &["rev-parse", "HEAD"])?;

        fs::write(repository.join(".gitignore"), "target/\n")?;
        fs::create_dir_all(repository.join(".github/workflows"))?;
        if include_workflow {
            fs::write(
                repository.join(".github/workflows/phase13-1-canonical-native.yml"),
                "name: fixture\non: workflow_dispatch\n",
            )?;
        }
        let helper = format!(
            r#"#!/usr/bin/env bash
set -euo pipefail
output=$1
candidate=$2
tree=$3
run_id=$4
mkdir -p "$output/canonical/logs"
printf 'canonical fixture\n' > "$output/canonical/logs/native.log"
if command -v sha256sum >/dev/null 2>&1; then
  digest=$(sha256sum "$output/canonical/logs/native.log" | awk '{{print $1}}')
else
  digest=$(shasum -a 256 "$output/canonical/logs/native.log" | awk '{{print $1}}')
fi
printf '%s  logs/native.log\n' "$digest" > "$output/canonical/logs.sha256"
jq -n --arg candidate "$candidate" --arg tree "$tree" --arg run "$run_id" \
  --arg tier "{}" '{{candidate_sha:$candidate,candidate_tree:$tree,workflow_run_id:$run,
  runner:{{os:"ubuntu-24.04",architecture:"x86_64"}},
  tools:{{rust:"1.97.0",clang:"22.1.8",cmake:"4.3.3",ninja:"1.13.2"}},
  evidence_tier:$tier,command_exits:[{{name:"native",exit_code:0}}],log_digests:"logs.sha256"}}' \
  > "$output/canonical/identity.json"
{}
"#,
            if helper_mode == "d2" { "D2" } else { "D1" },
            if helper_mode == "drift" {
                "printf 'drift\\n' >> source.txt\ngit add source.txt\ngit commit -q -m drift"
            } else {
                ":"
            }
        );
        write_executable(&repository.join("fixture-command.sh"), &helper)?;
        write_executable(
            &repository.join("fixture-prefix.sh"),
            r#"#!/usr/bin/env bash
set -euo pipefail
printf 'fixture-prefix\n' >> "${PHASE13_1_GAP_FAKE_COMMAND_JOURNAL:?}"
"#,
        )?;
        run_git(
            &repository,
            &[
                "add",
                ".gitignore",
                ".github",
                "fixture-command.sh",
                "fixture-prefix.sh",
            ],
        )?;
        run_git(&repository, &["commit", "-q", "-m", "candidate"])?;
        let candidate = run_git(&repository, &["rev-parse", "HEAD"])?;
        run_git(
            &repository,
            &[
                "init",
                "-q",
                "--bare",
                remote.to_str().ok_or("remote path is not UTF-8")?,
            ],
        )?;
        run_git(
            &repository,
            &[
                "remote",
                "add",
                "origin",
                remote.to_str().ok_or("remote path is not UTF-8")?,
            ],
        )?;
        run_git(&repository, &["push", "-q", "-u", "origin", "main"])?;

        let gh_journal = root.join("gh-journal.txt");
        let gh_state = root.join("gh-state.txt");
        write_executable(
            &fake_bin.join("gh"),
            r#"#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${PHASE13_1_GAP_FAKE_GH_JOURNAL:?}"
case "$*" in
  "repo view --json defaultBranchRef --jq .defaultBranchRef.name") printf 'main\n' ;;
  "repo view --json nameWithOwner --jq .nameWithOwner") printf 'fixture/repository\n' ;;
  "api repos/fixture/repository/actions/workflows/phase13-1-canonical-native.yml --jq .path")
    printf '.github/workflows/phase13-1-canonical-native.yml\n' ;;
  "workflow run phase13-1-canonical-native.yml --ref main -f candidate_sha="*)
    printf '%s\n' "${PHASE13_1_GAP_FAKE_DISPATCH_URL:-https://github.com/fixture/repository/actions/runs/7}" ;;
  "run view 7 --json databaseId,headSha,event,status,conclusion,url")
    count=0
    if [[ -f "${PHASE13_1_GAP_FAKE_GH_STATE:?}" ]]; then
      count=$(cat "${PHASE13_1_GAP_FAKE_GH_STATE}")
    fi
    count=$((count + 1))
    printf '%s\n' "$count" > "${PHASE13_1_GAP_FAKE_GH_STATE}"
    candidate=$(git rev-parse HEAD)
    if [[ "$count" -eq 1 ]]; then
      jq -cn --arg candidate "$candidate" '{databaseId:7,headSha:$candidate,event:"workflow_dispatch",status:"queued",conclusion:null,url:"https://github.com/fixture/repository/actions/runs/7"}'
    else
      jq -cn --arg candidate "$candidate" '{databaseId:7,headSha:$candidate,event:"workflow_dispatch",status:"completed",conclusion:"success",url:"https://github.com/fixture/repository/actions/runs/7"}'
    fi ;;
  "run watch 7 --exit-status") : ;;
  "run download 7 --name phase13-1-canonical-native-success-7-"*" --dir "*) : ;;
  "run list"*) printf '%s\n' "${PHASE13_1_GAP_FAKE_LISTING:-[]}" ;;
  *) printf 'unexpected gh call: %s\n' "$*" >&2; exit 1 ;;
esac
"#,
        )?;
        let tree = run_git(&repository, &["rev-parse", "HEAD^{tree}"])?;
        let manifest_json = json!({
            "schema":"phase13-1-gap-verification-manifest-v1",
            "test_fixture":true,
            "allowed_placeholders":["CANDIDATE","CANDIDATE_TREE","OUTPUT_ROOT","REMOTE_REF","CANONICAL_RUN_ID"],
            "structural_source":{"commit":candidate,"parent":parent},
            "deferred_xtask_targets":["phase13_acceptance_contract"],
            "artifacts":[{"id":"canonical-native","identity":"canonical/identity.json","logs":"canonical/logs","evidence_tier":"D1"}],
            "commands":[
                {"id":"fixture-prefix","argv":["bash","fixture-prefix.sh"],"environment":{},"stdout_log":"logs/fixture-prefix.stdout","stderr_log":"logs/fixture-prefix.stderr","evidence_class":"fixture"},
                {"id":"canonical-dispatch","argv":["gh","workflow","run","phase13-1-canonical-native.yml","--ref","${REMOTE_REF}","-f","candidate_sha=${CANDIDATE}"],"environment":{},"stdout_log":"logs/canonical-dispatch.stdout","stderr_log":"logs/canonical-dispatch.stderr","evidence_class":"canonical-d1"},
                {"id":"canonical-initial-view","argv":["gh","run","view","${CANONICAL_RUN_ID}","--json","databaseId,headSha,event,status,conclusion,url"],"environment":{},"stdout_log":"logs/canonical-initial-view.stdout","stderr_log":"logs/canonical-initial-view.stderr","evidence_class":"canonical-d1"},
                {"id":"canonical-watch","argv":["gh","run","watch","${CANONICAL_RUN_ID}","--exit-status"],"environment":{},"stdout_log":"logs/canonical-watch.stdout","stderr_log":"logs/canonical-watch.stderr","evidence_class":"canonical-d1"},
                {"id":"canonical-inspect","argv":["gh","run","view","${CANONICAL_RUN_ID}","--json","databaseId,headSha,event,status,conclusion,url"],"environment":{},"stdout_log":"logs/canonical-inspect.stdout","stderr_log":"logs/canonical-inspect.stderr","evidence_class":"canonical-d1"},
                {"id":"canonical-download","argv":["gh","run","download","${CANONICAL_RUN_ID}","--name","phase13-1-canonical-native-success-${CANONICAL_RUN_ID}-${CANDIDATE}","--dir","${OUTPUT_ROOT}/canonical"],"environment":{},"stdout_log":"logs/canonical-download.stdout","stderr_log":"logs/canonical-download.stderr","evidence_class":"canonical-d1"},
                {"id":"fixture","argv":["bash","fixture-command.sh","${OUTPUT_ROOT}","${CANDIDATE}","${CANDIDATE_TREE}","${CANONICAL_RUN_ID}"],"environment":{},"stdout_log":"logs/fixture.stdout","stderr_log":"logs/fixture.stderr","evidence_class":"fixture"}
            ],
            "fixture_tree":tree
        });
        let manifest = root.join("manifest.json");
        fs::write(&manifest, serde_json::to_vec_pretty(&manifest_json)?)?;
        Ok(Self {
            root,
            repository,
            manifest,
            fake_bin,
            candidate,
            gh_journal,
            gh_state,
            command_journal,
        })
    }

    fn run_with_settings(
        &self,
        candidate: &str,
        branch: &str,
        environment: &[(&str, &str)],
    ) -> TestResult<Output> {
        let path = format!("{}:{}", self.fake_bin.display(), env::var("PATH")?);
        let mut command =
            Command::new(workspace_root().join("scripts/phase13-1-gap-verification.sh"));
        command
            .args([candidate, "target/phase13-1-gap-verification", branch])
            .env("PATH", path)
            .env("PHASE13_1_GAP_MANIFEST", &self.manifest)
            .env("PHASE13_1_GAP_REPOSITORY_ROOT", &self.repository)
            .env("PHASE13_1_GAP_FAKE_GH_JOURNAL", &self.gh_journal)
            .env("PHASE13_1_GAP_FAKE_GH_STATE", &self.gh_state)
            .env("PHASE13_1_GAP_FAKE_COMMAND_JOURNAL", &self.command_journal);
        for (key, value) in environment {
            command.env(key, value);
        }
        Ok(command.output()?)
    }

    fn run_with(&self, candidate: &str, branch: &str) -> TestResult<Output> {
        self.run_with_settings(candidate, branch, &[])
    }

    fn run(&self) -> TestResult<Output> {
        self.run_with(&self.candidate, "main")
    }

    fn terminal_path(&self) -> PathBuf {
        self.repository
            .join("target/phase13-1-gap-verification")
            .join(&self.candidate)
            .join("final-verification.json")
    }

    fn assert_rejected_without_terminal(&self, output: &Output) {
        assert!(
            !output.status.success(),
            "producer unexpectedly accepted fixture"
        );
        assert!(
            !self.terminal_path().exists(),
            "failed producer published terminal evidence"
        );
    }

    fn mutate_manifest(&self, mutation: impl FnOnce(&mut Value)) -> TestResult {
        let mut manifest: Value = serde_json::from_slice(&fs::read(&self.manifest)?)?;
        mutation(&mut manifest);
        fs::write(&self.manifest, serde_json::to_vec_pretty(&manifest)?)?;
        Ok(())
    }
}

impl Drop for ProducerFixture {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).expect("owned producer fixture should be removable");
        }
    }
}

#[test]
fn evidence_validator_is_a_separate_fail_closed_executable() -> TestResult {
    // Arrange
    let source =
        fs::read_to_string(workspace_root().join("scripts/phase13-1-validate-gap-evidence.sh"))?;

    // Act / Assert
    assert!(source.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    assert!(!source.contains("source scripts/phase13-1-gap-verification.sh"));
    assert!(!source.contains("phase13-1-gap-verification.sh\""));
    assert!(source.contains("phase13-1-gap-verification-evidence-v1"));
    assert!(source.contains("evidence_tier"));
    assert!(source.contains("git merge-base --is-ancestor"));
    Ok(())
}

#[test]
fn producer_source_is_fail_closed_identity_last_and_publication_free() -> TestResult {
    // Arrange
    let source =
        fs::read_to_string(workspace_root().join("scripts/phase13-1-gap-verification.sh"))?;

    // Act / Assert
    assert!(source.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    assert!(source.contains("final-verification.json.pending"));
    assert!(source.contains("phase13-1-validate-gap-evidence.sh"));
    assert!(source.contains("mv -- \"$pending_path\" \"$terminal_path\""));
    assert!(!source.contains("git push"));
    assert!(!source.contains("force-push"));
    assert!(!source.contains("git update-ref"));
    assert!(!source.contains("eval "));
    assert!(!source.contains("RUST_TEST_THREADS"));
    Ok(())
}

#[test]
fn producer_publishes_only_validator_accepted_fixture_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "success")?;
    let output = fixture.run()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fixture.terminal_path().is_file());
    assert!(
        !fixture
            .terminal_path()
            .with_extension("json.pending")
            .exists()
    );
    Ok(())
}

#[test]
fn producer_rejects_wrong_candidate_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "success")?;
    let output = fixture.run_with("0000000000000000000000000000000000000000", "main")?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}

#[test]
fn producer_rejects_dirty_tree_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "success")?;
    fs::write(fixture.repository.join("dirty.txt"), "dirty\n")?;
    let output = fixture.run()?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}

#[test]
fn producer_rejects_non_default_branch_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "success")?;
    let output = fixture.run_with(&fixture.candidate, "develop")?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}

#[test]
fn producer_rejects_missing_remote_workflow_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(false, "success")?;
    let output = fixture.run()?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}

#[test]
fn producer_rejects_failed_command_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "success")?;
    fixture.mutate_manifest(|manifest| manifest["commands"][0]["argv"] = json!(["false"]))?;
    let output = fixture.run()?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}

#[test]
fn producer_rejects_missing_log_destination_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "success")?;
    fixture.mutate_manifest(|manifest| {
        manifest["commands"][0]["stdout_log"] = json!("missing/fixture.stdout");
    })?;
    let output = fixture.run()?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}

#[test]
fn producer_rejects_canonical_identity_drift_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "d2")?;
    let output = fixture.run()?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}

#[test]
fn producer_rejects_candidate_drift_without_terminal_evidence() -> TestResult {
    let fixture = ProducerFixture::new(true, "drift")?;
    let output = fixture.run()?;
    fixture.assert_rejected_without_terminal(&output);
    Ok(())
}
