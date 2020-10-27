#[test]
fn downstream_example ()
{
    let mut cmd = ::std::process::Command::new(env!("CARGO"));
    assert!(
        cmd .current_dir("downstream")
            .args(&["check", "--example", "user", "-q"])
            .status()
            .unwrap_or_else(|err| panic!(
                "Command `{:?}` failed: {}", cmd, err,
            ))
            .success()
        );
}
