use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Field<D: Ord + Hash + Copy, V: Clone> {
    default: V,
    overrides: HashMap<D, V>,
}

impl<D: Ord + Hash + Copy, V: Clone> Field<D, V> {
    pub(crate) fn new(default: V, overrides: HashMap<D, V>) -> Field<D, V> {
        Field { default, overrides }
    }

    pub fn local(&self) -> V {
        self.default.clone()
    }

    pub fn aligned_map<O, V2, F>(&self, other: &Field<D, V2>, transform: F) -> Field<D, O>
    where
        O: Clone,
        V2: Clone,
        F: Fn(&V, &V2) -> O,
        {
            Field::new(
                transform(&self.default, &other.default),
                self.overrides
                    .iter()
                    .filter_map(|(k, v)| {
                        other
                            .overrides
                            .get(k)
                            .map(|v2| (*k, transform(v, v2)))
                    })
                    .collect(),
                )
        }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field<D: Ord + Hash + Copy, V: Clone>(default: V, overrides: Vec<(D, V)>) -> Field<D, V> {
        Field::new(default, overrides.into_iter().collect())
    }

    #[test]
    fn test_local_returns_default() {
        let field = make_field(42u8, vec![(1u8, 100u8), (2u8, 200u8)]);
        assert_eq!(field.local(), 42u8);
    }

    #[test]
    fn test_aligned_map_basic() {
        let f1 = make_field(1u8, vec![(10u8, 2u8), (20u8, 3u8)]);
        let f2 = make_field(4u8, vec![(10u8, 5u8), (30u8, 6u8)]);
        // Only key 10u8 is present in both overrides
        let result = f1.aligned_map(&f2, |a, b| *a as u16 + *b as u16);

        // Default should be 1 + 4 = 5
        assert_eq!(result.local(), 5u16);

        // Only key 10u8 should be present in overrides, with value 2 + 5 = 7
        assert_eq!(result.overrides.len(), 1);
        assert_eq!(result.overrides.get(&10u8), Some(&7u16));
    }

    #[test]
    fn test_aligned_map_no_common_keys() {
        let f1 = make_field(1u8, vec![(10u8, 2u8)]);
        let f2 = make_field(4u8, vec![(20u8, 5u8)]);
        let result = f1.aligned_map(&f2, |a, b| *a as i32 - *b as i32);

        // Default should be 1 - 4 = -3
        assert_eq!(result.local(), -3i32);

        // No common keys, so overrides should be empty
        assert!(result.overrides.is_empty());
    }

    #[test]
    fn test_aligned_map_multiple_common_keys() {
        let f1 = make_field(0, vec![(1, 10), (2, 20), (3, 30)]);
        let f2 = make_field(100, vec![(2, 200), (3, 300), (4, 400)]);
        let result = f1.aligned_map(&f2, |a, b| a + b);

        // Default: 0 + 100 = 100
        assert_eq!(result.local(), 100);

        // Common keys: 2 and 3
        assert_eq!(result.overrides.len(), 2);
        assert_eq!(result.overrides.get(&2), Some(&(20 + 200)));
        assert_eq!(result.overrides.get(&3), Some(&(30 + 300)));
        assert!(!result.overrides.contains_key(&1));
        assert!(!result.overrides.contains_key(&4));
    }

    #[test]
    fn test_aligned_map_with_different_types() {
        let f1 = make_field("a", vec![(1, "b"), (2, "c")]);
        let f2 = make_field(10, vec![(1, 20), (2, 30)]);
        let result = f1.aligned_map(&f2, |s, n| format!("{}{}", s, n));

        assert_eq!(result.local(), "a10".to_string());
        assert_eq!(result.overrides.get(&1), Some(&"b20".to_string()));
        assert_eq!(result.overrides.get(&2), Some(&"c30".to_string()));
    }

    #[test]
    fn test_empty_overrides() {
        let f1: Field<i32, i32> = make_field(1, vec![]);
        let f2 = make_field(2, vec![]);
        let result = f1.aligned_map(&f2, |a, b| a * b);

        assert_eq!(result.local(), 2);
        assert!(result.overrides.is_empty());
    }
}