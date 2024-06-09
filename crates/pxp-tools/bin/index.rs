use std::env::args;

use discoverer::discover;
use indicatif::ProgressBar;
use pxp_index::Indexer;
use pxp_parser::parse;
use pxp_symbol::SymbolTable;
use rustyline::DefaultEditor;

fn main() {
    let args = args().skip(1).collect::<Vec<_>>();
    let directory = args.first().expect("error: no directory provided");
    let symbol_table = SymbolTable::the();
    let files = discover(&["php"], &[directory]).unwrap();
    let mut indexer = Indexer::new();

    println!("Indexing...");

    let bar = ProgressBar::new(files.len() as u64);

    for file in files.iter() {
        let result = parse(&std::fs::read(file).unwrap(), symbol_table);
        indexer.index(&result.ast);
        bar.inc(1);
    }

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("search: ");

        match readline {
            Ok(line) => match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                &["class", name] => {
                    let name = symbol_table.intern(name.as_bytes());
                    match indexer.get_index().get_class(name) {
                        Some(class) => {
                            dbg!(class);
                        },
                        None => println!("class not found."),
                    }
                },
                _ => {
                    println!("unrecognised input.");
                }
            },
            _ => panic!(),
        }
    }
}