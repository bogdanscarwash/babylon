//! Task 1 (#652 bsl-ls, PR A) — the pre-refactor golden-AST oracle
//! (plan §2.4). `read_all`'s parsed-tree shape is pinned, per file, for
//! every real `.bsl`/`.bscn` in the tree BEFORE the span-table refactor
//! touches `reader.rs`'s parser, so a change to what the reader *produces*
//! (as opposed to a read-only tap) shows up here rather than only in the
//! §5.6/tick-golden spot checks.
//!
//! **Corpus scope, and why it is walked rather than `include_str!`ed.**
//! Four directories hold every real BSL source file in this tree: the
//! shipped rule pack (`babylon-tick/content/rules/*.bsl`), this crate's
//! own conformance fixtures (`tests/conformance/*.bsl`), and the shipped
//! scenario corpus (`babylon-tick/content/scenarios/*.bscn` plus the
//! declaration preludes in `babylon-tick/content/declarations/*.bscn`). The plan
//! quoted 25/37/1 = 63 files, counted at planning time. Walking the
//! directories (sorted, for determinism — `read_dir` order is not
//! guaranteed) rather than hand-listing files means corpus drift is caught
//! here instead of silently under-covering the corpus again. Amendment AJ
//! retired the five author-visible `rng-draw` conformance fixtures; the
//! private finite-kernel draw boundary has typed contracts instead.
use babylon_bsl::reader::read_all;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// One content directory: its path (relative to this crate's manifest
/// directory) and the extension it contributes.
const CORPUS_DIRS: &[(&str, &str)] = &[
    ("../babylon-tick/content/rules", "bsl"),
    ("tests/conformance", "bsl"),
    ("../babylon-tick/content/scenarios", "bscn"),
    ("../babylon-tick/content/declarations", "bscn"),
];

fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let digest = Sha256::digest(bytes);
    digest.iter().fold(String::new(), |mut acc, byte| {
        let _ = write!(acc, "{byte:02x}");
        acc
    })
}

/// Every corpus file, sorted by filename WITHIN each directory (in the
/// fixed `CORPUS_DIRS` order) — deterministic regardless of the
/// filesystem's own `read_dir` ordering, and stable if a file is added to
/// or removed from one directory without touching the others' pins.
fn corpus_files() -> Vec<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for (dir, ext) in CORPUS_DIRS {
        let dir_path = manifest_dir.join(dir);
        let mut group: Vec<PathBuf> = std::fs::read_dir(&dir_path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", dir_path.display()))
            .map(|entry| entry.expect("readable dir entry").path())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some(*ext))
            .collect();
        group.sort();
        files.extend(group);
    }
    files
}

/// `(directory-relative label, pinned read_all digest)`, in `corpus_files`'
/// order. Filled in from a real run against the CURRENT (pre-refactor)
/// reader — the independent before/after oracle plan §2.4 describes. A
/// name mismatch means the corpus's membership moved; a hex mismatch means
/// `read_all`'s parsed-tree shape moved. Either is a STOP.
const PINNED_DIGESTS: &[(&str, &str)] = &[
    (
        "class-dynamics.bsl",
        "4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945",
    ),
    (
        "community.bsl",
        "2a6ae05861045b8505b356e71e92fb99bc0a0b9a80dce67c8e25ca6e4433b088",
    ),
    (
        "consciousness.bsl",
        "c98af42b4fb342dcc06c98d46b1a561778445095d026c45d6cdfdc3dce705652",
    ),
    (
        "control-ratio.bsl",
        "3ba50198b2f4e855a9fc50d3f5a9ec6fc0bae21db2d1585f785d77ab144815b3",
    ),
    (
        "decomposition.bsl",
        "1d5c1fc6efba986f540dd3a500dcf9f4aa0a7eb49a099d958a22ba995f286942",
    ),
    (
        "dispossession.bsl",
        "ccc325e7e91b8bbe0a0b904025b214c6138c6b2954263669c51b8581c93650a9",
    ),
    (
        "fundamental-theorem.bsl",
        "bf458f074a2ff0ef76cf96da7d1f31937165fd0a7b4e4021f809125e0c2766c9",
    ),
    (
        "imperial-rent.bsl",
        "2af8fa094edee81d04314b0ad9cc8ca6f3c41bb60e7bbf13a5b38cacc4b74099",
    ),
    (
        "lifecycle.bsl",
        "70aab772dab4c4122cfdb65d61839f0bb871d9af3e3f067ca49e473cf67ea9ef",
    ),
    (
        "metabolism.bsl",
        "02e0ba2e2e3212cfbe99031689aa483ebd39fee881027a5a7594f73a03c45409",
    ),
    (
        "organization.bsl",
        "4fca1533955051df5c8b4acc12cbf580dfbc4b85b87181e5b2675224b1139651",
    ),
    (
        "production.bsl",
        "7b50bce70a7ff8e85e22cc15f1dff7c5eb4baeddc89d86e14c0edd1ad8833ece",
    ),
    (
        "solidarity.bsl",
        "b6083aa048431ea1b9dedaf5ed9624595a366f8ee7afd0865c4766f2c9ca50d4",
    ),
    (
        "struggle-spark.bsl",
        "402a06d3c603537512c046c69a62e4a7aa1679c9f7ea6099bba6aa60d140ade0",
    ),
    (
        "territory.bsl",
        "0ae835ab2433ae529a620482e1d5ec72ef410a006c2178ca46603bbf97198197",
    ),
    (
        "vitality-attrition.bsl",
        "50621cfd0d628791bd5ade93daa81a23f029445bc356978b2b70869b24f1958e",
    ),
    (
        "vitality.bsl",
        "417d87f835f9bfaaa601dd09890580544d6e040ae69e0bafd9af3a7968210538",
    ),
    (
        "worldview.bsl",
        "e9ae9183db9bd82d4cef9b6563c2375a746d77bfc2f5302535318df71ea34957",
    ),
    (
        "doctrine_adventurism.bsl",
        "f1aa5ff4971b15c5aa1e96e2e85bc5675b4e725cf8004ff67d3f5b255e73230c",
    ),
    (
        "doctrine_liquidation_absorbing.bsl",
        "fc62da99af56446323275daf70b4007a63b02adb5fdd408aca0eae9ef8997b3b",
    ),
    (
        "doctrine_liquidationism.bsl",
        "0f296f8f33c54d35c03f6d2340150e35e6a67d159a830115d70f2f27b7930bfc",
    ),
    (
        "empty_when.bsl",
        "b194a865cd5d36ccc7f2676277ae92eef329415771e4044fc36374729b19e173",
    ),
    (
        "event_bifurcation.bsl",
        "1fcaab62d5a58747a7d1aa2974356d449236684ca2d52905d60e6e21fed7a458",
    ),
    (
        "event_edge_count.bsl",
        "2b45bb62fd408a10513db40c795d62c12061a503125d976305240f20069b1376",
    ),
    (
        "event_forall.bsl",
        "0db00c3affbbcde3b174acbdc14eee5d242967be297f45e8b5d2306cecd89a09",
    ),
    (
        "event_metric_conditions.bsl",
        "efd291da9c649ad623f203159df119dccc332136e5931f9eef4880ab7109ebad",
    ),
    (
        "event_node_condition.bsl",
        "61e5e16fabe20e48f96216382c5440c101ddb6065f7b8bee189ecc4e16ebe6c5",
    ),
    (
        "event_wealth_aggregates.bsl",
        "85c63b8d7ebf694ee9c49e4ee67eb60f6203d4d933fdb37b22b7162d5a713ce7",
    ),
    (
        "unconditional.bsl",
        "3086eb8e3e6aa89bb704bd369872c4434826b00c6d80630c3f34ce27e9110c8a",
    ),
    (
        "unknown_metric.bsl",
        "83559f3c1fe5339ff3f0948f389e1b7b1d36697c1a32cc88d40707d0c44749cc",
    ),
    (
        "carceral-arc-conformance.bscn",
        "d1b4577f98813cfd17b5e8bd394918a56c5e822c051790bc045254cf6b2a13ad",
    ),
    (
        "class-dynamics-conformance.bscn",
        "864fb0ade8cfd61c0fdaa32db4569b6443e6cd8ce225812cbed24d28f5481c28",
    ),
    (
        "community-carrier-collision-conformance.bscn",
        "fbc3bdb51d979dbcb24b46ce4d40c6d2dfdb830cb6b36e4ba30bbd4642961424",
    ),
    (
        "community-conformance.bscn",
        "5c23e7783683d6c8b3fdb4d57bf2427c219405dfacbd35bdf44d51fc1adbb610",
    ),
    (
        "community-cost-modifier-conformance.bscn",
        "113927d5c2bd4f3b767ba5836bf21dbe217d0cece25c2542b68a68bbf957ec28",
    ),
    (
        "community-decay-arc-conformance.bscn",
        "466fdb9e06026b6e92c011e977af7fd2d82491af1d3258d6d59331ce7ff915f1",
    ),
    (
        "community-degenerate-conformance.bscn",
        "7d41b8c88b92b7c7a38d28810ccc58c4588a7c2ac1b5c935c951c28f91a0d4d8",
    ),
    (
        "community-empty-conformance.bscn",
        "818d6347f95820f750c72eb2b986da6d2c34f3a1dba4dbd302fd80d0890f83fd",
    ),
    (
        "community-floor-conformance.bscn",
        "affd8f131a56d76a8d467c86096892bc35c2e732aa5f5e21e8a38c040485bf55",
    ),
    (
        "community-solidarity-seam-conformance.bscn",
        "013c3754d216e6ac19608de8dbfafe710a195d0c5747e4774ffdc9c32947a77e",
    ),
    (
        "community-tie-conformance.bscn",
        "78c31a62d6208c93cefe9edabf770ed26aafb6308b1b214aaa2e12a475cdd9e7",
    ),
    (
        "consciousness-ternary-conformance.bscn",
        "e2b5d4991107fd9c7de01f8294bc602d7fe0a6a5a3ba3d062c23d74e8764a2a4",
    ),
    (
        "control-ratio-conformance.bscn",
        "dad3f4a61f7e433a682340a8a7c3f5f62725552d0c98e1c76b191114642ab64a",
    ),
    (
        "control-ratio-revolution-conformance.bscn",
        "57c40b939b24018b67fd01c811b52c4c103a06e372b16df35d489e1a1f71fe01",
    ),
    (
        "control-ratio-within-capacity-conformance.bscn",
        "71767fa10014db7e38abee1b4ee8b884cb7873009a688f8555eb7a9f8406c68c",
    ),
    (
        "control-ratio-zero-enforcer-conformance.bscn",
        "32d3cc3e7bff6b42909fdd77ee5709aa98a5adb85436e17e7f41d84650c1b105",
    ),
    (
        "decomposition-conformance.bscn",
        "9604a3dd733dca5e19ac3c7054d0bcee365e12cba98f45b311b0e61ef1dfbb72",
    ),
    (
        "decomposition-delay-conformance.bscn",
        "38629b42c921b772c50392c262db0cd7c83a3a706ed0749fd0bf5e0783530172",
    ),
    (
        "dispossession-ceiling-matrix-conformance.bscn",
        "f54909f01f5063f6484bbd4f91cd918a8e1db162d85af18b876046867fc04ef5",
    ),
    (
        "dispossession-conformance.bscn",
        "f5bfe458cc19b288afabb131ae954eabb48f263f96d97a7672ca955030932b57",
    ),
    (
        "dispossession-negative-input-conformance.bscn",
        "d39cf58c3161ae6ffdce2d445fdc9e20794678f40192a862e48264cbbbb7bdc1",
    ),
    (
        "dispossession-negative-weight-conformance.bscn",
        "2a4e78aaa6ab5c3beee1c321239a0d9d463479be3e5336be4348b3fe77e10c4f",
    ),
    (
        "dispossession-saturation-conformance.bscn",
        "b99361f2442a5dea483bc50e64bd385f33e105a3ad751a8dd561b39346fd7676",
    ),
    (
        "dispossession-single-rate-conformance.bscn",
        "561fdea375b900df8b4c12a2d39ada31e4cd57753b673d095ee2f06fa18fbc33",
    ),
    (
        "dispossession-zero-rate-conformance.bscn",
        "f51f85e92ef9acc659c9f10743562f28706f5d62662ce1a7a191bb176a08dcbc",
    ),
    (
        "edge-lane-e2e.bscn",
        "2349d230c64ba90225946e9b85334752f9c2beb5a6cb759a8b85635e43a0f800",
    ),
    (
        "edge-write-lane-e2e.bscn",
        "81fd0c95579c372ff02a32b4c8bc15e46579b488ee8093b6248e14d974ff36d8",
    ),
    (
        "imperial-rent-conformance.bscn",
        "5480badcd818c2b629420ae33562f9c2b590b8eff8d197b279544dd152768355",
    ),
    (
        "imperial-rent-multi-tribute-conformance.bscn",
        "55af1c7bcb4c9332c2b5845c7d622da955321edaea8d962c37f28d20c0f2210b",
    ),
    (
        "lifecycle-conformance.bscn",
        "d194d575355b6f30e446c892dd74b470fbcc3d3aaf9610d078fee6f93636b7a7",
    ),
    (
        "lifecycle-crisis-conformance.bscn",
        "754295542d9aaa3df0ebb1c805e632ea2a892ecc156bd4e9b0a7d36dc5fb74a4",
    ),
    (
        "lifecycle-zero-pop-p-conformance.bscn",
        "58bb3e5aa1ba9147a53c54ed2bc0a81a649fdaaf9dfc4e92d7e7b3f3a7e861af",
    ),
    (
        "metabolism-ceiling-conformance.bscn",
        "a7dd2c1e19633a3ac1e9f76730f39d4362615b6e32c5089ce0ff832d08526468",
    ),
    (
        "metabolism-ceiling-suppression-conformance.bscn",
        "2d4f0cb353ed980f10741a1c69d93cff4ba39698de64f7c2d8f892f73445e0ab",
    ),
    (
        "metabolism-conformance.bscn",
        "6050080ce553ae5916fcdc8ef23f83acaac3b1ec341cecae93e8f2a6702b5972",
    ),
    (
        "metabolism-entropy-high-conformance.bscn",
        "5c7e0ed77f6c866036870d8e09f9777187c02bfee6404c199e6fe2033d7144ec",
    ),
    (
        "metabolism-entropy-low-conformance.bscn",
        "bd08cdd45d41e4b44f38c13297232ddef0b0bcc6d10ec45282b185c2cf690a04",
    ),
    (
        "metabolism-extreme-damage-conformance.bscn",
        "b6995b06d7009056970b5dbc25eb3d246371c7326bb181fe308f4fc6d6dbf5ca",
    ),
    (
        "metabolism-ratcheted-ceiling-conformance.bscn",
        "996ed84bc534e80c93de5b0d8a91deed7bdf61cce9a23bcb15cb79abc512a9ce",
    ),
    (
        "metabolism-rounding-divergence-conformance.bscn",
        "04ae00c60aabb359788a2f96fa294b78cd5432300fef99495ae51dd2a0063615",
    ),
    (
        "organization-foundation.bscn",
        "fa287abb29a4f611e2bafda2cd6d06c0ff5c1f8896c4cff2291928d722fa58f5",
    ),
    (
        "organization-practice-contract.bscn",
        "8307e32fea7bd667ed8994b281bcf6f6d5f67170ea190edc6bc811e49903f159",
    ),
    (
        "production-conformance.bscn",
        "4179010e9abe89641c1713891ffd0867aa1d26588d9c4e96d51e7ef072423721",
    ),
    (
        "query-lane-e2e.bscn",
        "d6cd61b2fedb9d80ca24ba4366f0f96c7bb57969007170c7d740fe5a1069d407",
    ),
    (
        "solidarity-conformance.bscn",
        "251b7bf2843f2a6d3db25bb79c822be177903bc4258cfd48c8caa56cd46d6223",
    ),
    (
        "struggle-spark-conformance.bscn",
        "25876a9eb6c5c83288022535100035c9c9a993cb8fa9b64675283bc9957ce825",
    ),
    (
        "territory-conformance.bscn",
        "2c0bf3c8a7a6d67ac4dacc7d24b5c1e8801075b725198cd207f0486035e2fc40",
    ),
    (
        "two-classes.bscn",
        "278918747d41b041853c13a1c31dfb13e34ff8315a2db6441a3a35834624c76e",
    ),
    (
        "us-counties-lifecycle-demo.bscn",
        "75af712c5d5562adf78c1d169852159c521732c4939333de5719fea58d8d6d17",
    ),
    (
        "vitality-attrition-conformance.bscn",
        "5d54b8aa59ad350af6764541549e5e51bd719e3ecc28b43f7eeb5719baff48f8",
    ),
    (
        "vitality-conformance.bscn",
        "b61ffb7e5d64a6d88eeee1808200b7c0d724c364dc61e34fabf49e4ee4a11620",
    ),
    (
        "vitality-lifecycle-combined-conformance.bscn",
        "bef48841b30fe8fd32a142fd316d6cf8a3e357ef7252ba0ce3e9d0867cf44a0f",
    ),
    (
        "worldview-foundation.bscn",
        "e1ad5e1a5a0fafa858d398cdca5202b8897041329f60a8a03f6370b4b2e8e40d",
    ),
    (
        "organization-practice.bscn",
        "7c3809a3e36d238c0bd7abc329d378305fb1b517918fdecf2f9fbbce23e0b1f8",
    ),
    (
        "worldview.bscn",
        "23a650f0f3bdb33da8e8bc66f6aa8d27475ddc8422adbcb93637f66cf3d26ae0",
    ),
];

#[test]
fn read_all_digest_is_pinned_across_the_full_content_corpus() {
    let files = corpus_files();
    let mut actual: Vec<(String, String)> = Vec::with_capacity(files.len());
    for path in &files {
        let label = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("utf8 filename")
            .to_string();
        let bytes =
            std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let forms = read_all(&bytes)
            .unwrap_or_else(|e| panic!("{} failed to read_all: {e:?}", path.display()));
        let digest = sha256_hex(format!("{forms:#?}").as_bytes());
        actual.push((label, digest));
    }

    let corpus_drift = actual.len() == PINNED_DIGESTS.len()
        && actual.iter().zip(PINNED_DIGESTS).any(
            |((label, digest), (expected_label, expected_digest))| {
                label != expected_label || digest != expected_digest
            },
        );
    if actual.len() != PINNED_DIGESTS.len() || corpus_drift {
        use std::fmt::Write as _;
        let mut dump = String::new();
        for (label, digest) in &actual {
            let _ = writeln!(dump, "    (\"{label}\", \"{digest}\"),");
        }
        panic!(
            "PINNED_DIGESTS is stale ({} pinned vs {} discovered) — \
             paste this into PINNED_DIGESTS:\n{dump}",
            PINNED_DIGESTS.len(),
            actual.len()
        );
    }

    for ((label, digest), (expected_label, expected_digest)) in actual.iter().zip(PINNED_DIGESTS) {
        assert_eq!(
            label, expected_label,
            "content corpus membership/order moved — re-derive PINNED_DIGESTS deliberately"
        );
        assert_eq!(
            digest, expected_digest,
            "read_all's parsed tree for {label} moved"
        );
    }
}
