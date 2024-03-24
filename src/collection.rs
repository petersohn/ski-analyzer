use std::collections::HashMap;
use std::hash::Hash;

pub fn max_if<It, F, P, B>(it: It, mut func: F, mut pred: P) -> Option<It::Item>
where
    It: Iterator,
    F: FnMut(&It::Item) -> B,
    P: FnMut(&It::Item, &B) -> bool,
    B: PartialOrd,
{
    it.filter_map(|item| {
        let value = func(&item);
        if pred(&item, &value) {
            Some((item, value))
        } else {
            None
        }
    })
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    .map(|item| item.0)
}

pub fn get_or_default<'a, K, V>(map: &'a mut HashMap<K, V>, key: K) -> &'a mut V
where
    K: Clone + Eq + Hash,
    V: Default,
{
    let val = map.get_mut(&key);
    let ret = match val {
        Some(value) => value,
        None => {
            map.insert(key.clone(), V::default());
            map.get_mut(&key).unwrap()
        }
    };
}
