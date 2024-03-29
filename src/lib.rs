use std::{fmt::Debug, mem::MaybeUninit};

pub struct SlabMap<K, V, const N: usize>
where
    K: Ord,
{
    slots: [MaybeUninit<Record<K, V>>; N],
    len: usize,
    tail: Option<Box<SlabMap<K, V, N>>>,
}

impl<K, V, const N: usize> Clone for SlabMap<K, V, N>
where
    K: Ord + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        let mut map = Self {
            slots: unsafe { MaybeUninit::uninit().assume_init() },
            len: self.len,
            tail: self.tail.clone(),
        };

        let slots =
            &unsafe { std::mem::transmute::<_, &[Record<K, V>]>(&self.slots[..]) }[..self.len];

        for (i, item) in slots.iter().enumerate() {
            let item = item.clone();
            map.slots[i] = MaybeUninit::new(item);
        }

        map
    }
}

impl<K, V, const N: usize> Debug for SlabMap<K, V, N>
where
    K: Ord + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Default for SlabMap<K, V, 8_usize>
where
    K: Ord,
{
    fn default() -> Self {
        SlabMap {
            slots: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
            tail: None,
        }
    }
}

impl<K: Ord, V, const N: usize> SlabMap<K, V, N> {
    pub fn new() -> SlabMap<K, V, N> {
        SlabMap {
            slots: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
            tail: None,
        }
    }
}

pub struct Record<K, V> {
    key: K,
    value: V,
}

impl<K, V> Clone for Record<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

impl<K: Ord, V, const N: usize> Drop for SlabMap<K, V, N> {
    fn drop(&mut self) {
        for item in &mut self.slots[..self.len] {
            unsafe { std::ptr::drop_in_place(item) }
        }
    }
}

pub struct Iter<'a, K: Ord, V, const N: usize> {
    map: &'a SlabMap<K, V, N>,
    cur: usize,
}

impl<'a, K: Ord, V, const N: usize> Iterator for Iter<'a, K, V, N> {
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
        let r = unsafe { &*self.map.slots[cur].as_ptr() };
        return Some((&r.key, &r.value));
    }
}

impl<K: Ord, V, const N: usize> SlabMap<K, V, N> {
    #[inline]
    fn new_tail(record: Record<K, V>) -> Box<Self> {
        let mut map = Self {
            slots: unsafe { MaybeUninit::uninit().assume_init() },
            len: 1,
            tail: None,
        };
        map.slots[0] = MaybeUninit::new(record);
        Box::new(map)
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V, N> {
        Iter { map: self, cur: 0 }
    }

    #[inline]
    pub fn binary_search<'a>(&'a self, key: &K) -> Result<usize, Option<usize>> {
        let v = &self.slots[0..self.len].binary_search_by(|x| unsafe { &*x.as_ptr() }.key.cmp(key));

        match v {
            Ok(v) => Ok(*v),
            Err(v) if *v >= N => Err(None),
            Err(v) => Err(Some(*v)),
        }
    }

    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        let mut t = Some(self);
        while let Some(slab) = t {
            match slab.binary_search(key) {
                Ok(_) => return true,
                Err(_) => {
                    t = slab.tail.as_ref().map(|x| &**x);
                }
            }
        }
        false
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut t = Some(self);
        while let Some(slab) = t {
            match slab.binary_search(key) {
                Ok(i) => return Some(&unsafe { &*slab.slots[i].as_ptr() }.value),
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
    ) -> Result<Option<V>, (K, V)> {
        let mut is_fallback = None;

        match self.binary_search(&key) {
            Ok(i) => {
                std::mem::swap(
                    &mut unsafe { &mut *self.slots[i].as_mut_ptr() }.value,
                    &mut value,
                );
                return Ok(Some(value));
            }
            Err(sorted_index) => {
                if let Some(index) = sorted_index {
                    if !has_fallback {
                        is_fallback = Some(index)
                    }
                }

                if let Some(v) = &mut self.tail {
                    match v.insert_inner(key, value, has_fallback || is_fallback.is_some()) {
                        Ok(v) => return Ok(v),
                        Err((k, v)) => {
                            key = k;
                            value = v;
                        }
                    }
                }
            }
        }

        if let Some(index) = is_fallback {
            if index < std::cmp::min(N, self.len) {
                self.slots[index..self.len].rotate_right(1);
            } else {
                self.len += 1;
            }
            self.slots[index] = MaybeUninit::new(Record { key, value });
            return Ok(None);
        } else if !has_fallback {
            self.tail = Some(Self::new_tail(Record { key, value }));
            return Ok(None);
        }

        return Err((key, value));
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.insert_inner(key, value, false) {
            Ok(v) => v,
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
