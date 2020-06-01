use std::{mem::ManuallyDrop, fmt::Debug};

const CAPACITY: usize = 8;

pub struct SlabMap<K, V>
where
    K: Ord,
{
    slots: [ManuallyDrop<Record<K, V>>; CAPACITY],
    len: usize,
    tail: Option<Box<SlabMap<K, V>>>,
}

impl<K: Ord + Debug, V: Debug> Debug for SlabMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}

impl<K: Ord, V> Default for SlabMap<K, V> {
    fn default() -> Self {
        SlabMap {
            slots: unsafe { std::mem::zeroed() },
            len: 0,
            tail: None,
        }
    }
}

struct Record<K, V> {
    key: K,
    value: V,
}

impl<K: Ord, V> Drop for SlabMap<K, V> {
    fn drop(&mut self) {
        let mut record: ManuallyDrop<Record<K, V>> = unsafe { std::mem::zeroed() };
        for slot in self.slots.iter_mut() {
            std::mem::swap(&mut record, slot);
            unsafe { ManuallyDrop::drop(&mut record) };
        }
    }
}

pub struct Iter<'a, K: Ord, V> {
    map: &'a SlabMap<K, V>,
    cur: usize
}

impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.map.len {
            if let Some(tail) = self.map.tail.as_ref() {
                self.map = &*tail;
                self.cur = 0;
                return self.next();
            } else {
                return None;
            }
        }

        let cur = self.cur;
        self.cur += 1;
        let r = &self.map.slots[cur];
        return Some((&r.key, &r.value));
    }
}

impl<K: Ord, V> SlabMap<K, V> {
    fn new_tail(record: Record<K, V>) -> Box<Self> {
        let mut map = Self {
            slots: unsafe { std::mem::zeroed() },
            len: 1,
            tail: None,
        };
        map.slots[0] = ManuallyDrop::new(record);
        Box::new(map)
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter { map: self, cur: 0 }
    }

    #[inline]
    pub fn binary_search<'a>(&'a self, key: &K) -> Result<usize, Option<usize>> {
        let v = &self.slots[0..self.len].binary_search_by(|x| x.key.cmp(key));

        match v {
            Ok(v) => Ok(*v),
            Err(v) if *v == CAPACITY => {
                Err(None)
            }
            Err(v) => {
                Err(Some(*v))
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut t = Some(self);
        while let Some(slab) = t {
            match slab.binary_search(key) {
                Ok(i) => return Some(&slab.slots[i].value),
                Err(_) => {
                    t = slab.tail.as_ref().map(|x| &**x);
                }
            }
        }
        None
    }

    #[inline(always)]
    pub fn insert_inner<'a>(&'a mut self, mut key: K, mut value: V, has_fallback: bool) -> Result<(), (K, V)> {
        let mut is_fallback = None;

        match self.binary_search(&key) {
            Ok(i) => {
                self.slots[i].value = value;
                return Ok(());
            },
            Err(sorted_index) => {
                if let Some(index) = sorted_index {
                    if !has_fallback {
                        is_fallback = Some(index)
                    }
                }

                if let Some(v) = &mut self.tail {
                    if let Err((k, v)) = v.insert_inner(key, value, has_fallback || is_fallback.is_some()) {
                        key = k;
                        value = v;
                    } else {
                        return Ok(());
                    }
                }
            }
        }

        if let Some(index) = is_fallback {
            if index < self.len {
                // for i in (index..self.len).rev() {
                //     self.slots.swap(i + 1, i);
                // }
                // for (index + 1)
                unsafe { std::ptr::copy(&self.slots[index], &mut self.slots[index + 1], self.len - index) };
            }
            self.slots[index] = ManuallyDrop::new(Record { key, value });
            self.len += 1;
            return Ok(());
        } else if !has_fallback {
            self.tail = Some(Self::new_tail(Record { key, value }));
            return Ok(());
        }

        return Err((key, value))
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.insert_inner(key, value, false) {
            Ok(_) => {},
            Err(_) => unreachable!("failed to insert")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut map = SlabMap::default();

        for i in 0..1000 {
            map.insert(i, i.to_string());
        }

        println!("{:?}", map);
    }
}