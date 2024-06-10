use discoverer::discover;
use pxp_index::{Index, Indexer};
use pxp_parser::parse;
use pxp_symbol::{Symbol, SymbolTable};
use pxp_type::Type;

#[test]
fn it_indexes_classes() {
    let index = index();

    let a = index.get_class(name(b"A")).unwrap();

    assert!(a.get_name() == name(b"A"));
    assert!(a.get_short_name() == name(b"A"));
    assert!(a.get_namespace() == None);

    let b = index.get_class(name(b"B")).unwrap();

    assert!(b.get_name() == name(b"B"));
    assert!(b.get_short_name() == name(b"B"));
    assert!(b.get_namespace() == None);
    
    let parent = b.get_parent().unwrap();
    assert!(parent.get_name() == name(b"A"));

    let c = index.get_class(name(b"C")).unwrap();

    assert!(c.get_name() == name(b"C"));
    assert!(c.get_short_name() == name(b"C"));
    assert!(c.get_namespace() == None);

    let parent = c.get_parent().unwrap();
    assert!(parent.get_name() == name(b"B"));

    // FIXME: Add in tests here for interfaces.

    let properties = index.get_class(name(b"Properties")).unwrap();

    assert!(properties.get_name() == name(b"Properties"));
    assert!(properties.get_short_name() == name(b"Properties"));
    assert!(properties.get_namespace() == None);

    let properties_properties = properties.get_properties().collect::<Vec<_>>();

    assert!(properties_properties.len() == 6);

    let a = properties.get_property(name(b"a")).unwrap();
    assert!(a.is_public());
    assert!(a.get_type() == &Type::Mixed);

    let b = properties.get_property(name(b"b")).unwrap();
    assert!(b.is_public());
    assert!(b.get_type() == &Type::String);

    let c = properties.get_property(name(b"c")).unwrap();
    assert!(c.is_protected());
    assert!(c.get_type() == &Type::Integer);

    let d = properties.get_property(name(b"d")).unwrap();
    assert!(d.is_private());
    assert!(d.get_type() == &Type::Boolean);

    let e = properties.get_property(name(b"e")).unwrap();
    assert!(e.is_static());

    let f = properties.get_property(name(b"f")).unwrap();
    assert!(f.is_public());
    assert!(f.get_type() == &Type::Mixed);

    let methods = index.get_class(name(b"Methods")).unwrap();

    assert!(methods.get_name() == name(b"Methods"));
    assert!(methods.get_short_name() == name(b"Methods"));
    assert!(methods.get_namespace() == None);

    let methods_methods = methods.get_methods().collect::<Vec<_>>();

    assert!(methods_methods.len() == 5);

    let a = methods.get_method(name(b"a")).unwrap();
    assert!(a.is_public());
    assert!(a.get_return_type() == &Type::Mixed);

    let b = methods.get_method(name(b"b")).unwrap();
    assert!(b.is_public());
    assert!(b.get_return_type() == &Type::String);

    let c = methods.get_method(name(b"c")).unwrap();
    assert!(c.is_protected());
    assert!(c.get_return_type() == &Type::Integer);

    let d = methods.get_method(name(b"d")).unwrap();
    assert!(d.is_private());
    assert!(d.get_return_type() == &Type::Boolean);

    let e = methods.get_method(name(b"e")).unwrap();
    assert!(e.is_static());
}

#[test]
fn it_indexes_functions() {
    let index = index();

    let a = index.get_function(name(b"a")).unwrap();

    assert!(a.get_name() == name(b"a"));
    assert!(a.get_short_name() == name(b"a"));
    assert!(a.get_namespace() == None);

    let c = index.get_function(name(b"c")).unwrap();

    assert!(c.get_name() == name(b"c"));
    assert!(c.get_short_name() == name(b"c"));
    assert!(c.get_namespace() == None);

    let c_parameters = c.get_parameters().collect::<Vec<_>>();

    assert!(c_parameters.len() == 1);
    assert!(c_parameters[0].get_name() == name(b"a"));
    assert!(c_parameters[0].get_type() == &Type::String);

    let d = index.get_function(name(b"d")).unwrap();

    assert!(d.get_return_type() == &Type::Integer);

    let e = index.get_function(name(b"e")).unwrap();

    assert!(e.returns_by_reference());

    let b = index.get_function(name(b"A\\b")).unwrap();

    assert!(b.get_name() == name(b"A\\b"));
    assert!(b.get_short_name() == name(b"b"));
    assert!(b.get_namespace() == Some(name(b"A")));
}

fn name(name: &[u8]) -> Symbol {
    SymbolTable::the().intern(name)
}

fn index() -> Index {
    let mut indexer = Indexer::new();
    let files = discover(&["php"], &["tests/fixtures"]).expect("failed to discover files");

    for file in files.iter() {
        let result = parse(&std::fs::read(&file).expect("failed to read file"), SymbolTable::the());
        indexer.index(&result.ast);
    }

    indexer.get_index().clone()
}