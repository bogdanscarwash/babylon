//! Fixture crate for bsl-lint's I1 ordering-pin test: this crate directory
//! is deliberately created BEFORE `crate-alpha` (see the sibling crate's doc
//! comment) so that on filesystems where `std::fs::read_dir` reflects
//! creation order for small directories, an unsorted file walk would list
//! this crate first — even though `crate-alpha` sorts first alphabetically.
//! `list_src_rs_files` must sort so the finding's evidence string and
//! `(file, line)` header always cite `crate-alpha` before `crate-zulu`,
//! regardless of directory-iteration order.

/// Real emission site — shares `"E-FAKE-555"` with crate-alpha, deliberately
/// unallowlisted (the ordering-pin RED fixture; distinct from the
/// `E-FAKE-777` pair used by `an_unallowlisted_cross_file_e_code_fails`).
pub fn classify(x: i32) -> &'static str {
    match x {
        0 => "E-FAKE-555",
        _ => "E-FAKE-001",
    }
}
