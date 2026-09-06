/// Find the closest match to a string in a list of strings.
pub fn find_closest<'a>(s: &str, compare: &[&'a str]) -> Option<&'a str> {
    let mut best_confidence = 0.8; // minimum confidence
    let mut best_match = None;
    for valid in compare {
        let confidence = strsim::jaro_winkler(s, valid);
        if confidence > best_confidence {
            best_confidence = confidence;
            best_match = Some(*valid);
        }
    }
    best_match
}

/// Find the closest match to a string in a list of elements, removing it.
pub fn take_closest<T: std::fmt::Display>(s: &str, compare: &mut Vec<T>) -> Option<T> {
    let mut best_confidence = 0.8; // minimum confidence
    let mut best_index = None;
    for (i, valid) in compare.iter().enumerate() {
        let confidence = strsim::jaro_winkler(s, &valid.to_string());
        if confidence > best_confidence {
            best_confidence = confidence;
            best_index = Some(i);
        }
    }
    best_index.map(|index| compare.remove(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_closest_basic() {
        let options = ["apple", "banana", "cherry", "date"];

        assert_eq!(find_closest("appl", &options), Some("apple"));
        assert_eq!(find_closest("bannana", &options), Some("banana"));
        assert_eq!(find_closest("cheri", &options), Some("cherry"));
        assert_eq!(find_closest("dat", &options), Some("date"));

        assert_eq!(find_closest("xyz", &options), None);
    }

    #[test]
    fn take_closest_basic() {
        let mut options = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "date".to_string(),
        ];

        assert_eq!(
            take_closest("appl", &mut options),
            Some("apple".to_string())
        );
        assert_eq!(take_closest("appl", &mut options), None); // already taken

        assert_eq!(
            take_closest("bannana", &mut options),
            Some("banana".to_string())
        );
        assert_eq!(
            take_closest("cheri", &mut options),
            Some("cherry".to_string())
        );
        assert_eq!(take_closest("dat", &mut options), Some("date".to_string()));

        assert_eq!(take_closest("xyz", &mut options), None);
    }
}
