use obs_c_oracle as _;

use std::collections::BTreeSet;
use std::fs;

/// Names of fns following a `#[unsafe(no_mangle)]` / `#[no_mangle]` attribute.
fn no_mangle_fns(src: &str) -> Vec<String> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t != "#[unsafe(no_mangle)]" && t != "#[no_mangle]" {
            continue;
        }
        for next in &lines[i + 1..] {
            if let Some(pos) = next.find("fn ") {
                let name: String = next[pos + 3..]
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                out.push(name);
                break;
            }
        }
    }
    out
}

/// Plain C identifiers (no `*`, not `rust_eh_personality`) between `local:`
/// and `};` of a linker version script, ignoring `/* */` comments.
fn hidden_names(map: &str) -> BTreeSet<String> {
    let mut text = String::new();
    let mut rest = map;
    while let Some(start) = rest.find("/*") {
        text.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        rest = after.find("*/").map_or("", |end| &after[end + 2..]);
    }
    text.push_str(rest);

    let local = text
        .split_once("local:")
        .expect("rust-exports.map has a local: section")
        .1;
    let local = local.split("};").next().unwrap_or(local);
    local
        .split(';')
        .map(str::trim)
        .filter(|n| !n.is_empty() && !n.contains('*') && *n != "rust_eh_personality")
        .map(String::from)
        .collect()
}

#[test]
fn rust_exports_list_matches_no_mangle_shims() {
    let list = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../libobs/cmake/rust-exports.txt"
    ))
    .expect("read rust-exports.txt");
    let listed: BTreeSet<String> = list
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();

    // Plain C names under `local:` in rust-exports.map: shims for symbols the
    // C libobs does not export (e.g. utf8.c), hidden instead of listed.
    let map = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../libobs/cmake/rust-exports.map"
    ))
    .expect("read rust-exports.map");
    let hidden = hidden_names(&map);

    let mut found = BTreeSet::new();
    let mut visited = Vec::new();
    for krate in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).expect("read rust/") {
        let ffi_dir = krate.expect("dir entry").path().join("src/ffi");
        if !ffi_dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&ffi_dir).expect("read src/ffi") {
            let path = entry.expect("dir entry").path();
            if path.extension().is_some_and(|e| e == "rs") {
                let src = fs::read_to_string(&path).expect("read ffi source");
                found.extend(no_mangle_fns(&src));
            }
        }
        visited.push(ffi_dir);
    }
    for krate in ["obs-util", "obs-graphics"] {
        assert!(
            visited
                .iter()
                .any(|d| d.ends_with(format!("{krate}/src/ffi"))),
            "{krate}/src/ffi was not scanned: {visited:?}"
        );
    }

    let both: Vec<_> = listed.intersection(&hidden).collect();
    assert!(
        both.is_empty(),
        "in both rust-exports.txt and the rust-exports.map local: list: {both:?}"
    );

    let expected: BTreeSet<String> = listed.union(&hidden).cloned().collect();
    let missing_from_list: Vec<_> = found.difference(&expected).collect();
    let missing_from_src: Vec<_> = expected.difference(&found).collect();
    assert!(
        missing_from_list.is_empty() && missing_from_src.is_empty(),
        "shims neither in rust-exports.txt nor hidden in rust-exports.map: \
         {missing_from_list:?}; listed or hidden but no #[no_mangle] shim in \
         rust/*/src/ffi:{missing_from_src:?}"
    );
}
