#![allow(deprecated)]

use core::borrow::Borrow;
use core::fmt::{Debug, Formatter};
use core::hash::{BuildHasher, Hash, Hasher};
use core::hash::SipHasher13;
use spinlock::SpinNoIrq;

static PARK_MILLER_LEHMER_SEED: SpinNoIrq<u32> = SpinNoIrq::new(0);
const RAND_MAX: u64 = 2_147_483_647;

pub fn random() -> u64 {
    let mut seed = PARK_MILLER_LEHMER_SEED.lock();
    if *seed == 0 {
        *seed = arceos_api::time::ax_current_time().as_secs() as u32;
    }

    let mut ret: u128 = 0;
    for _ in 0..4 {
        *seed = ((u64::from(*seed) * 48271) % RAND_MAX) as u32;
        ret = (ret << 32) | (*seed as u128);
    }
    ret as u64
}

pub struct RandomState {
    k0: u64,
    k1: u64,
}

impl RandomState {
    pub fn new() -> Self {
        Self {
            k0: random(),
            k1: random(),
        }
    }
}

impl Default for RandomState {
    #[inline]
    fn default() -> Self {
        RandomState::new()
    }
}

impl Debug for RandomState {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RandomState")
            .field("k0", &self.k0)
            .field("k1", &self.k1)
            .finish()
    }
}

impl BuildHasher for RandomState {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher(SipHasher13::new_with_keys(self.k0, self.k1))
    }
}

pub struct DefaultHasher(SipHasher13);

impl DefaultHasher {
    fn new() -> Self {
        Self(SipHasher13::new_with_keys(0, 0))
    }
}

impl Default for DefaultHasher {
    #[inline]
    fn default() -> Self {
        DefaultHasher::new()
    }
}

impl Hasher for DefaultHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }

    #[inline]
    fn write_str(&mut self, s: &str) {
        self.0.write_str(s)
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    base: hashbrown::hash_map::Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
}

pub struct HashMap<K, V, S = RandomState> {
    base: hashbrown::HashMap<K, V, S>,
}

impl<K, V, S> HashMap<K, V, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            base: hashbrown::HashMap::with_hasher(hash_builder),
        }
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            base: self.base.iter(),
        }
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.base.insert(k, v)
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.base.get(k)
    }

    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.base.get_mut(k)
    }

    pub fn remove<Q: ?Sized>(&mut self, k: &K) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.base.remove(k)
    }

    pub fn clear(&mut self) {
        self.base.clear()
    }
}

impl<K, V> HashMap<K, V, RandomState> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    S: Default,
{
    #[inline]
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}