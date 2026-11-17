use std::fs;
use std::path::Path;

/// A source in a git checkout normalizes like a registry source of the same
/// package, which a checkout's path does not name: the package is read from
/// the nearest manifest inside the checkout.
#[test]
fn test() {
    let home = std::env::temp_dir().join(format!("trybuild-cargo-git-{}", std::process::id()));
    let checkout = home.join(".cargo/git/checkouts/nami-3b7bff5da959545f/8217e30");
    let package = checkout.join("core");
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(
        checkout.join("Cargo.toml"),
        "[workspace]\nmembers = [\"core\"]\n",
    )
    .unwrap();
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"nami-core\"\nversion.workspace = true\n",
    )
    .unwrap();
    let file = package.join("src/id.rs");
    fs::write(&file, "").unwrap();

    let original = format!(
        "
error[E0277]: `Row` has no stable identity
  --> tests/ui/identifiable.rs:16:20
   |
16 |     for_each(rows, |row| text(row.name))
   |              ^^^^ expected a type implementing `Identifiable`
   |
help: the following other types implement trait `Identifiable`
  --> {}:91:1
   |
91 |   impl<T: Ord + Clone> Identifiable for SelfId<T> {{
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `SelfId<T>`
",
        file.display(),
    );
    let expected = "
error[E0277]: `Row` has no stable identity
  --> tests/ui/identifiable.rs:16:20
   |
16 |     for_each(rows, |row| text(row.name))
   |              ^^^^ expected a type implementing `Identifiable`
   |
help: the following other types implement trait `Identifiable`
  --> $CARGO/nami-core-$VERSION/src/id.rs
   |
   |   impl<T: Ord + Clone> Identifiable for SelfId<T> {
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `SelfId<T>`
";

    let context = crate::normalize::Context {
        krate: "trybuild000",
        input_file: Path::new("tests/ui/identifiable.rs"),
        source_dir: &crate::directory::Directory::new("/git/trybuild/test_suite"),
        workspace: &crate::directory::Directory::new("/git/trybuild"),
        target_dir: &crate::directory::Directory::new("/git/trybuild/target"),
        path_dependencies: &[],
    };
    let preferred = crate::normalize::diagnostics(&original, context)
        .preferred()
        .to_owned();
    fs::remove_dir_all(&home).unwrap();
    assert_eq!(preferred, expected);
}
