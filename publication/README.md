# Publication migration provenance

This is a separately sanitized publishing history, not a rewrite of the accepted local `codex/experiment-012` branch. Original HEAD remains `f13bb0998a3bf6f3e63299e276015becf1452aa5`. The original repository is retained at `C:/Users/Aaron/CoderVibe/SocialDeduction`.

A verified backup at `C:/Users/Aaron/CoderVibe/SocialDeduction-publication-backup-f13bb09-20261010` contains the complete all-ref Git bundle, a restored mirror verified with `git fsck --full`, and byte-identical copies of all 19 evidence artifacts. [Backup manifest](backup-manifest.json) records the bundle hash, every artifact hash and the inventory of all original oversized blobs, including historical objects.

[commit-map.json](commit-map.json) maps all 35 original commits to the sanitized history. The migration verified every corresponding file tree, excluding only the six stripped evidence files, and verified author/committer identities, timestamps and messages. No simulation source, test, dependency, toolchain, scientific result or frozen JSON content changed. Original platform evidence retains its historical source revisions; they are provenance, not assertions about the new publishing commit ID.

The six oversized artifacts live unchanged in [release 409465057](https://github.com/Eren-Enou/Social_Deduction/releases/tag/frozen-evidence-006-012-f13bb09). All six were uploaded, their API size/digest identities checked, then independently downloaded into this initially artifact-free publishing checkout using the committed retrieval mechanism. All 19 files matched their original byte lengths and SHA-256 digests. Other thirteen artifacts stay in Git. The release is not configured for GitHub's repository-level immutable-release enforcement; fixed numeric IDs and SHA-256 make changed/deleted/replaced objects fail retrieval.

The history was exported/imported only in an isolated repository. The first import failed on a separator following an omitted blob; its diagnostic repository was retained, and a fresh import corrected that separator. No accepted original ref or evidence was modified. `publication-backup.py` and `publication-sanitize.py` record the executed migration procedure; their recorded paths are for this migration, not a command to rerun destructively over an existing backup.

Six existing replay workflows now retrieve and verify evidence and run the 12 retrieval tests before their unchanged Rust gates. Missing evidence remains a failure; no frozen regression was weakened. Evidence files are ignored only in the publishing checkout so retrieval cannot accidentally recommit oversized blobs.

Local publication validation and final delivery status are recorded in `validation.md` after the complete migration gate succeeds. No publishing branch push or PR is authorized before that validation completes. No Experiment 013 is included, and final merge requires owner approval.
