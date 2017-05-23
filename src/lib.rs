extern crate filebuffer;
extern crate goblin;

use goblin::{parse, Object, elf, archive};
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::iter::FromIterator;

pub fn needed_libraries<'a, R>(map: &'a BTreeMap<&'a str, BTreeSet<&'a str>>, roots: R) -> Vec<&'a str>
    where R: IntoIterator<Item=&'a str>
{
    struct Result<'a> {
        map: &'a BTreeMap<&'a str, BTreeSet<&'a str>>,
        libs: Vec<&'a str>,
        seen: HashSet<&'a str>,
    }

    impl<'a> Result<'a> {
        fn go(&mut self, root: &'a str) {
            if self.seen.insert(root) {
                if let Some(deps) = self.map.get(root) {
                    for dep in deps.iter() {
                        self.go(dep);
                    }
                }
                self.libs.push(root);
            }
        }
    }

    let mut result = Result {
        map: map,
        libs: Vec::new(),
        seen: HashSet::new(),
    };
    for root in roots {
        result.go(root);
    }

    result.libs.reverse();
    result.libs
}

pub fn dependency_map<'a, N>(fnames: N) -> BTreeMap<&'a str, BTreeSet<&'a str>>
    where N: IntoIterator<Item=&'a str>
{
    let bufs: Vec<_> = fnames.into_iter().filter_map(|fname| {
        if let Ok(buf) = filebuffer::FileBuffer::open(fname) {
            Some((fname, buf))
        } else {
            None
        }
    }).collect();
    let bufrefs: Vec<_> = bufs.iter().map(|&(fname, ref buf)| (fname, buf as &[u8])).collect();

    let mut defined = HashMap::new();
    let needed: Vec<_> = bufrefs.iter().filter_map(|&(fname, ref buf)| {
        let (obj_defined, obj_needed) = match parse(buf) {
            Ok(Object::Elf(elf)) => elf_symbols(elf),
            Ok(Object::Archive(archive)) => archive_symbols(archive, buf),
            _ => return None,
        };
        for sym in obj_defined {
            defined.entry(sym).or_insert_with(HashSet::new).insert(fname);
        }
        Some((fname, obj_needed))
    }).collect();

    needed.into_iter().map(|(fname, obj_needed)| {
        let mut seen = BTreeSet::new();
        for sym in obj_needed {
            if let Some(choices) = defined.get(sym) {
                if !choices.iter().any(|tgt| seen.contains(tgt)) {
                    // TODO: if choices.len() > 1, do something clever
                    seen.extend(choices.iter());
                }
            }
        }
        (fname, seen)
    }).collect()
}

type Syms<'a> = Vec<&'a str>;

fn elf_symbols<'a>(obj: elf::Elf<'a>) -> (Syms<'a>, Syms<'a>) {
    (elf_symbols_group(&obj, true), elf_symbols_group(&obj, false))
}

fn elf_symbols_group<'a>(obj: &elf::Elf<'a>, want_defined: bool) -> Syms<'a> {
    obj.syms.iter()
        .filter(|sym| {
            elf::sym::st_bind(sym.st_info) != elf::sym::STB_LOCAL &&
            match elf::sym::st_type(sym.st_info) {
                elf::sym::STT_OBJECT | elf::sym::STT_FUNC => want_defined,
                elf::sym::STT_NOTYPE => !want_defined,
                _ => false,
            }
        })
        .map(|sym| obj.strtab.get(sym.st_name).expect("symbol not in string table"))
        .collect()
}

fn archive_symbols<'a>(obj: archive::Archive<'a>, bufref: &'a &[u8]) -> (Syms<'a>, Syms<'a>) {
    let mut defined = Vec::new();
    let mut needed = HashSet::new();
    for member in obj.members() {
        if let Ok(buf) = obj.extract(member, bufref) {
            if let Ok(Object::Elf(elf)) = parse(buf) {
                let (member_defined, member_needed) = elf_symbols(elf);
                defined.extend(member_defined);
                needed.extend(member_needed);
            }
        }
    }
    for sym in defined.iter() {
        needed.remove(sym);
    }
    (defined, Vec::from_iter(needed))
}
