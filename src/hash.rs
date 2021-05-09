const NHASH: usize = 32;
const MULTIPLIER: usize = 31;

use std::mem;

#[derive(Clone, Debug)]
pub struct NameVal<T: Clone> {
    name: String,
    value: T,
}

pub struct Hash<T: Clone> {
    table: Vec<Vec<NameVal<T>>>,
    bits: usize,
    split_bucket: usize,
}

pub fn bit_string(bits: usize) -> String {
    let mut bstr: String = "".to_string();
    for i in 0..(mem::size_of::<usize>()*8) {
        let mask = 1 << i;
        bstr.push(if mask & bits == 0 { '0' } else { '1' });
    }

    bstr.chars().rev().collect()
}

impl<T> Hash<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        let mut hash_vec = Vec::with_capacity(NHASH);

        for _i in 0..NHASH {
            hash_vec.push(Vec::new());
        }

        Hash {
            table: hash_vec,
            bits: 5, // log_2(32)
            split_bucket: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    fn hash(&self, name: &str) -> usize {
        let mut h: usize = 0;

        for p in name.bytes() {
            let p = p as usize;
            // Instead of silently wrapping (like most C implementations do,
            // even if that is strictly undefined), rust panics if we overflow
            // an integer value. So we need to use this magic instead.
            h = h.wrapping_mul(MULTIPLIER).wrapping_add(p);
        }

        let m = h & ((1 << self.bits) - 1);
        if m < self.len() {
            m
        } else {
            m ^ (1 << (self.bits - 1))
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&T> {
        let h = self.hash(name);
        let entries = &self.table[h];

        match entries.len() {
            0 => None,
            _ => {
                for entry in entries.iter() {
                    if entry.name == name {
                        return Some(&entry.value);
                    }
                }
                None
            }
        }
    }

    // Returns true if the insert was a new key,
    // False if we overwrote a key
    pub fn upsert(&mut self, name: &str, value: T) -> bool {
        let h = self.hash(name);
        let entry_count;

        {
            let entries = &mut self.table[h];
            entry_count = entries.len();

            for entry in entries.iter_mut() {
                if entry.name == name {
                    entry.value = value;
                    return false;
                }
            }

            entries.push(NameVal {
                name: name.to_string(),
                value: value,
            });
        }

        if entry_count + 1 > (1 << self.bits) {
            self.split();
        }

        true
    }

    fn split(&mut self) {
        let orig_bucket = self.table[self.split_bucket].to_vec(); 
        self.table[self.split_bucket] = Vec::<NameVal<T>>::new(); 
        self.table.push(Vec::<NameVal<T>>::new());

        if self.len() > ((1 << self.bits) - 1) {
            self.bits += 1;
            self.split_bucket = 0;
        } else {
            self.split_bucket += 1;
        }

        for entry in orig_bucket.iter() {
            self.upsert(&entry.name, entry.value.clone());
        }
    }

    pub fn remove(&mut self, name: &str) -> Option<T> {
        let h = self.hash(name);
        let entries = &mut self.table[h];

        for i in 0..entries.len() {
            if entries[i].name == name {
                return Some(entries.remove(i).value);
            }
        }

        None
    }
}

#[test]
fn basics() {
    let mut hashtab = Hash::new();
    assert_eq!(hashtab.lookup("abc"), None);
    hashtab.upsert("abc", 64);
    hashtab.upsert("abcdefghijklmnopq", 128);
    assert_eq!(hashtab.lookup("abc"), Some(&64));
    assert_eq!(hashtab.lookup("abc"), Some(&64));
    assert_eq!(hashtab.lookup("abcdefghijklmnopq"), Some(&128));
    hashtab.upsert("abc", 256);
    assert_eq!(hashtab.lookup("abc"), Some(&256));
    hashtab.remove("abc");
    assert_eq!(hashtab.lookup("abc"), None);
    assert_eq!(hashtab.lookup("abcd"), None);
    let nippon = "私はガラスを食べられます。それは私を傷つけません。";
    hashtab.upsert(nippon, 31337);
    assert_eq!(hashtab.lookup(nippon), Some(&31337));
    hashtab.remove(nippon);
    assert_eq!(hashtab.lookup(nippon), None);
    println!("done");
}

#[test]
fn test_split() {
    let mut hashtab = Hash::new();

    let n_entries = 8192;

    for i in 0..n_entries {
        let k = i.to_string();
        hashtab.upsert(&k, i);
    }

    for i in 0..n_entries {
        let k = i.to_string();
        let val = hashtab.lookup(&k);

        assert!(val.is_some());
        let unwrapped = val.unwrap();
        assert_eq!(*unwrapped, i);
    }
}
