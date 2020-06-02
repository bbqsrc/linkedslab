use std::{fmt::Debug, mem::ManuallyDrop};
use generic_array::{ArrayLength, GenericArray};
use generic_array::typenum::{self, Unsigned};

// const CAPACITY: usize = 12;

pub struct SlabMap<K, V, N = typenum::U12>
where
    K: Ord,
    N: ArrayLength<ManuallyDrop<Record<K, V>>>
{
    slots: GenericArray<ManuallyDrop<Record<K, V>>, N>, //[ManuallyDrop<Record<K, V>>; CAPACITY],
    len: usize,
    tail: Option<Box<SlabMap<K, V, N>>>,
}

impl<K: Ord + Debug, V: Debug, N: ArrayLength<ManuallyDrop<Record<K, V>>>> Debug for SlabMap<K, V, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
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


impl<K: Ord, V, N: ArrayLength<ManuallyDrop<Record<K, V>>>> SlabMap<K, V, N> {
    pub fn new() -> SlabMap<K, V, N> {
        SlabMap {
            slots: unsafe { std::mem::zeroed() },
            len: 0,
            tail: None,
        }
    }
}

pub struct Record<K, V> {
    key: K,
    value: V,
}

impl<K: Ord, V, N: ArrayLength<ManuallyDrop<Record<K, V>>>> Drop for SlabMap<K, V, N> {
    fn drop(&mut self) {
        let mut record: ManuallyDrop<Record<K, V>> = unsafe { std::mem::zeroed() };
        for slot in self.slots.iter_mut() {
            std::mem::swap(&mut record, slot);
            unsafe { ManuallyDrop::drop(&mut record) };
        }
    }
}

pub struct Iter<'a, K: Ord, V, N: ArrayLength<ManuallyDrop<Record<K, V>>>> {
    map: &'a SlabMap<K, V, N>,
    cur: usize,
}

impl<'a, K: Ord, V, N: ArrayLength<ManuallyDrop<Record<K, V>>>> Iterator for Iter<'a, K, V, N> {
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

impl<K: Ord, V, N: ArrayLength<ManuallyDrop<Record<K, V>>>> SlabMap<K, V, N> {
    #[inline]
    fn new_tail(record: Record<K, V>) -> Box<Self> {
        let mut map = Self {
            slots: unsafe { std::mem::zeroed() },
            len: 1,
            tail: None,
        };
        map.slots[0] = ManuallyDrop::new(record);
        Box::new(map)
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V, N> {
        Iter { map: self, cur: 0 }
    }

    #[inline]
    pub fn binary_search<'a>(&'a self, key: &K) -> Result<usize, Option<usize>> {
        let v = &self.slots[0..self.len].binary_search_by(|x| x.key.cmp(key));

        match v {
            Ok(v) => Ok(*v),
            Err(v) if *v >= <N as Unsigned>::to_usize() => Err(None),
            Err(v) => Err(Some(*v)),
        }
    }

    #[inline]
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
    pub fn insert_inner<'a>(
        &'a mut self,
        mut key: K,
        mut value: V,
        has_fallback: bool,
    ) -> Result<(), (K, V)> {
        let mut is_fallback = None;

        match self.binary_search(&key) {
            Ok(i) => {
                self.slots[i].value = value;
                return Ok(());
            }
            Err(sorted_index) => {
                if let Some(index) = sorted_index {
                    if !has_fallback {
                        is_fallback = Some(index)
                    }
                }

                if let Some(v) = &mut self.tail {
                    if let Err((k, v)) =
                        v.insert_inner(key, value, has_fallback || is_fallback.is_some())
                    {
                        key = k;
                        value = v;
                    } else {
                        return Ok(());
                    }
                }
            }
        }

        if let Some(index) = is_fallback {
            if index < std::cmp::min(<N as Unsigned>::to_usize(), self.len) {
                self.slots[index..self.len].rotate_right(1);
            } else {
                self.len += 1;
            }
            self.slots[index] = ManuallyDrop::new(Record { key, value });
            return Ok(());
        } else if !has_fallback {
            self.tail = Some(Self::new_tail(Record { key, value }));
            return Ok(());
        }

        return Err((key, value));
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        match self.insert_inner(key, value, false) {
            Ok(_) => {}
            Err(_) => unreachable!("failed to insert"),
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
