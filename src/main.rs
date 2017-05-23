extern crate static_ldd;

use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let fnames;
    let roots;
    if let Some(dashdash) = args.iter().position(|arg| arg == "--") {
        let (before, after) = args.split_at(dashdash);
        fnames = before.iter().chain(after.iter().skip(1));
        roots = before.iter();
    } else {
        fnames = args.iter().chain([].iter().skip(0));
        roots = args.iter();
    }

    let deps = static_ldd::dependency_map(fnames.map(|f| f as &str));

    for (lib1, libs) in deps.iter() {
        for lib2 in libs {
            println!("{}\t{}", lib1, lib2);
        }
    }

    for lib in static_ldd::needed_libraries(&deps, roots.map(|f| f as &str)) {
        println!("{}", lib);
    }
}
