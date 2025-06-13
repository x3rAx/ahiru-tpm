use std::{collections::HashSet, hash::Hash};

pub fn dedup_by_key<T, K>(items: Vec<T>, key: impl Fn(&T) -> K) -> Vec<T>
where
    K: Eq + Hash,
{
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(key(item)))
        .collect()
}

pub fn ensure_unique_by_key<T, K>(items: &[T], key: impl Fn(&T) -> K) -> Result<(), &T>
where
    K: Eq + Hash,
{
    let mut seen = HashSet::new();
    for item in items {
        if !seen.insert(key(item)) {
            return Err(item);
        }
    }
    Ok(())
}

pub fn ensure_unique<T: Eq + Hash + Clone>(items: Vec<T>) -> Result<Vec<T>, T> {
    let mut seen = HashSet::new();
    let mut output = Vec::with_capacity(items.len());

    for item in items {
        if !seen.insert(item.clone()) {
            return Err(item);
        }
        output.push(item);
    }

    Ok(output)
}

pub fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "yes" | "y" | "on" | "1" => Some(true),
        "false" | "no" | "n" | "off" | "0" => Some(false),
        _ => None,
    }
}
