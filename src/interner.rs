use std::collections::HashMap;

use bumpalo::Bump;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Symbol(usize);

pub(crate) struct StringInterner {
    interned_strs: HashMap<&'static str, Symbol>,
    indexed_strs: Vec<&'static str>,
    buffer: Bump,
}

impl Default for StringInterner {
    fn default() -> StringInterner {
        StringInterner::new()
    }
}

impl StringInterner {
    pub(crate) fn new() -> StringInterner {
        StringInterner {
            interned_strs: Default::default(),
            indexed_strs: Default::default(),
            buffer: Default::default(),
        }
    }

    pub(crate) fn get_or_intern(&mut self, string: &str) -> Symbol {
        use std::collections::hash_map::RawEntryMut;

        let string_hash = {
            use std::hash::{BuildHasher, Hasher};

            let mut hasher = self.interned_strs.hasher().build_hasher();
            hasher.write_str(string.as_ref());

            hasher.finish()
        };

        let symbol = Symbol(self.interned_strs.len());

        let entry = self
            .interned_strs
            .raw_entry_mut()
            .from_key_hashed_nocheck(string_hash, string);

        match entry {
            RawEntryMut::Occupied(e) => *e.get(),
            RawEntryMut::Vacant(e) => {
                let buffed_string = {
                    let allocated_str = self.buffer.alloc_str(string);

                    unsafe {
                        // SAFETY: The memory allocated by `buffer` will live for as long as the
                        // program does, so, for practical purposes, it is essentially static.
                        &*(allocated_str as *const str)
                    }
                };

                e.insert_hashed_nocheck(string_hash, buffed_string, symbol);
                self.indexed_strs.push(buffed_string);

                symbol
            }
        }
    }

    pub(crate) fn resolve(&self, symbol: Symbol) -> &'static str {
        unsafe {
            // SAFETY: All symbols are guaranteed to have been created by us, so there's no
            // need to check whether the symbol is valid.
            self.indexed_strs.get_unchecked(symbol.0)
        }
    }
}
