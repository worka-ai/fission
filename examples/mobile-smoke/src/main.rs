#[cfg(target_os = "android")]
fn main() {}

#[cfg(target_os = "ios")]
fn main() -> anyhow::Result<()> {
    mobile_smoke::run_mobile()
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
fn main() -> anyhow::Result<()> {
    mobile_smoke::run_desktop()
}
