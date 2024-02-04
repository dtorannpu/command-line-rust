use assert_cmd::Command;

#[test]
fn works() {
    let mut cmd = Command::new("hello");
    let res = cmd.output();

    match res {
        Ok(ref output) => println!("{}", String::from_utf8_lossy(&output.stdout)),
        _ => {}
    }
    assert!(res.is_ok());
}

#[test]
fn runs() {
    let mut cmd = Command::cargo_bin("hello").unwrap();
    cmd.assert().success().stdout("Hello, world!!!\n");
}

#[test]
fn true_ok() {
    let mut cmd = Command::cargo_bin("true").unwrap();
    cmd.assert().success();
}

#[test]
fn false_not_ok() {
    let mut cmd = Command::cargo_bin("false").unwrap();
    cmd.assert().failure();
}