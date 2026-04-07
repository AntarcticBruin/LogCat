use super::types::{DirEntry, EntryKind};

pub fn is_probably_text_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".log")
        || lower.ends_with(".txt")
        || lower.ends_with(".md")
        || lower.ends_with(".csv")
        || lower.ends_with(".json")
        || lower.ends_with(".xml")
        || lower.ends_with(".sh")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".conf")
        || lower.ends_with(".ini")
        || lower.ends_with(".js")
        || lower.ends_with(".ts")
        || lower.ends_with(".py")
        || lower.ends_with(".rs")
        || lower.ends_with(".go")
        || lower.ends_with(".toml")
        || lower.ends_with(".env")
}

pub fn is_probably_binary_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".gz")
        || lower.ends_with(".zip")
        || lower.ends_with(".tar")
        || lower.ends_with(".tgz")
        || lower.ends_with(".bz2")
        || lower.ends_with(".xz")
        || lower.ends_with(".7z")
        || lower.ends_with(".so")
        || lower.ends_with(".dll")
        || lower.ends_with(".exe")
        || lower.ends_with(".bin")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".pdf")
}

pub fn should_probe_text_file(path: &str) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    !file_name.contains('.')
}

pub fn sort_entries(items: &mut [DirEntry]) {
    items.sort_by(|left, right| {
        if left.kind == right.kind {
            left.name.to_lowercase().cmp(&right.name.to_lowercase())
        } else {
            match (&left.kind, &right.kind) {
                (EntryKind::Dir, _) => std::cmp::Ordering::Less,
                (_, EntryKind::Dir) => std::cmp::Ordering::Greater,
                (EntryKind::File, _) => std::cmp::Ordering::Less,
                (_, EntryKind::File) => std::cmp::Ordering::Greater,
                (EntryKind::Symlink, _) => std::cmp::Ordering::Less,
                (_, EntryKind::Symlink) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        }
    });
}
