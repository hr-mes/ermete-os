fn main() {}
fn test() -> Result<bool, String> {
    let ok: Result<Result<bool, String>, String> = Ok(Ok(true));
    if true {
        ok?
    } else {
        Err(" error\.into())
