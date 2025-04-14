use std::env;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        let pkg_name = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "Unknown".to_string());
        let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
        let pkg_authors = env::var("CARGO_PKG_AUTHORS").unwrap_or_else(|_| "Unknown".to_string());
        let pkg_description = env::var("CARGO_PKG_DESCRIPTION").unwrap_or_else(|_| "".to_string());
        res.set("FileDescription", &pkg_description);
        res.set("ProductName", &pkg_name);
        res.set("OriginalFilename", &format!("{}.exe", pkg_name));
        res.set("CompanyName", &pkg_authors);
        res.set("LegalCopyright", "Copyright © 2025");
        let version_parts: Vec<u16> = pkg_version
            .split('.')
            .take(3)
            .map(|s| {
                s.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u16>()
                    .unwrap_or(0)
            })
            .collect();

        let (major, minor, patch, revision) = match version_parts.as_slice() {
            [major, minor, patch] => (*major, *minor, *patch, 0),
            [major, minor] => (*major, *minor, 0, 0),
            _ => (0, 0, 0, 0),
        };
        let major_minor = ((major as u32) << 16) | (minor as u32);
        let patch_revision = ((patch as u32) << 16) | (revision as u32);
        let version_number = ((major_minor as u64) << 32) | (patch_revision as u64);
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, version_number);
        res.set_version_info(winres::VersionInfo::FILEVERSION, version_number);
        res.set_icon("icon.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
