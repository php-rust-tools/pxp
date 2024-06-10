use pxp_span::Span;
use pxp_symbol::{Symbol, SymbolTable};
use pxp_type::Type;
use pxp_visitor::{walk_braced_namespace, walk_class_statement, walk_unbraced_namespace, Visitor};
use pxp_ast::{UnbracedNamespace, *};

use crate::{class_like::{ClassLike, Method}, function::Function, parameter::Parameter, Index};

#[derive(Debug, Clone)]
pub struct Indexer {
    index: Index,
    context: IndexerContext,
}

impl Indexer {
    pub fn new() -> Self {
        Indexer {
            index: Index::new(),
            context: IndexerContext::default(),
        }
    }

    pub fn index(&mut self, ast: &[Statement]) {
        self.visit(ast);
    }

    pub fn get_index(&self) -> &Index {
        &self.index
    }

    fn transform_function_parameter_list(&self, parameters: &FunctionParameterList) -> Vec<Parameter> {
        parameters.parameters.iter().map(|p| {
            let name = p.name.stripped;
            let r#type = p.data_type.as_ref().map(|r| r.get_type()).unwrap_or_else(|| &Type::Mixed).clone();
            let default = p.default.is_some();
            let variadic = p.ellipsis.is_some();
            let reference = p.ampersand.is_some();
            
            Parameter { name, r#type, default, variadic, reference }
        }).collect()
    }

    fn transform_constructor_parameter_list(&self, parameters: &ConstructorParameterList) -> Vec<Parameter> {
        parameters.parameters.iter().map(|p| {
            let name = p.name.symbol;
            let r#type = p.data_type.as_ref().map(|r| r.get_type()).unwrap_or_else(|| &Type::Mixed).clone();
            let default = p.default.is_some();
            let variadic = p.ellipsis.is_some();
            let reference = p.ampersand.is_some();
            
            Parameter { name, r#type, default, variadic, reference }
        }).collect()
    }
}

#[derive(Debug, Clone, Default)]
struct IndexerContext {
    namespace: Option<Symbol>,
    class: Option<ClassLike>,
}

impl IndexerContext {
    fn namespace(&self) -> Option<Symbol> {
        self.namespace
    }

    fn class(&mut self) -> &mut ClassLike {
        self.class.as_mut().unwrap()
    }

    fn in_class(&self) -> bool {
        self.class.is_some()
    }

    fn set_class(&mut self, class: ClassLike) {
        self.class = Some(class);
    }
}

impl Visitor for Indexer {
    fn visit_unbraced_namespace(&mut self, node: &UnbracedNamespace) {
        self.context.namespace = Some(node.name.as_resolved().unwrap().resolved);
        walk_unbraced_namespace(self, node);
        self.context.namespace = None;
    }

    fn visit_braced_namespace(&mut self, node: &BracedNamespace) {
        self.context.namespace = node.name.as_ref().map(|n| n.as_resolved().unwrap().resolved);
        walk_braced_namespace(self, node);
        self.context.namespace = None;
    }

    fn visit_function_statement(&mut self, node: &FunctionStatement) {
        let name = node.name.as_resolved().unwrap().resolved;
        let short = node.name.as_resolved().unwrap().original;
        let namespace = self.context.namespace();
        let parameters = self.transform_function_parameter_list(&node.parameters);
        let return_type = node.return_type.as_ref().map(|r| r.data_type.get_type()).unwrap_or_else(|| &Type::Mixed).clone();
        let returns_by_reference = node.ampersand.is_some();

        self.index.add_function(Function { name, short, namespace, parameters, return_type, returns_by_reference });
    }

    fn visit_class_statement(&mut self, node: &ClassStatement) {
        let name = node.name.as_resolved().unwrap();

        let mut class = ClassLike::new(name.resolved, name.original, self.context.namespace());
        class.parent = node.extends.as_ref().map(|e| e.parent.as_resolved().unwrap().resolved);
        class.interfaces = node.implements.as_ref().map(|i| i.interfaces.iter().map(|i| i.as_resolved().unwrap().resolved).collect::<Vec<_>>()).unwrap_or_else(|| Vec::new());

        self.context.set_class(class);
        walk_class_statement(self, node);
        
        let class = self.context.class.as_ref().unwrap().clone();

        self.index.add_class(class);
        self.context.class = None;
    }

    fn visit_concrete_method(&mut self, node: &ConcreteMethod) {
        if !self.context.in_class() {
            return;
        }

        let name = node.name.symbol;
        let return_type = node.return_type.as_ref().map(|r| r.data_type.get_type()).unwrap_or_else(|| &Type::Mixed).clone();
        let modifiers = node.modifiers.clone();
        let parameters = self.transform_function_parameter_list(&node.parameters);

        self.context.class().methods.push(Method { name, return_type, modifiers, parameters, r#abstract: false });
    }

    fn visit_abstract_method(&mut self, node: &AbstractMethod) {
        if !self.context.in_class() {
            return;
        }

        let name = node.name.symbol;
        let return_type = node.return_type.as_ref().map(|r| r.data_type.get_type()).unwrap_or_else(|| &Type::Mixed).clone();
        let modifiers = node.modifiers.clone();
        let parameters = self.transform_function_parameter_list(&node.parameters);

        self.context.class().methods.push(Method { name, return_type, modifiers, parameters, r#abstract: true });
    }

    fn visit_concrete_constructor(&mut self, node: &ConcreteConstructor) {
        if !self.context.in_class() {
            return;
        }

        let name = SymbolTable::the().intern(b"__construct");
        let return_type = Type::Void;
        let modifiers = node.modifiers.clone();
        let parameters = self.transform_constructor_parameter_list(&node.parameters);

        self.context.class().methods.push(Method { name, return_type, modifiers, parameters, r#abstract: false });
    }

    fn visit_abstract_constructor(&mut self, node: &AbstractConstructor) {
        if !self.context.in_class() {
            return;
        }

        let name = SymbolTable::the().intern(b"__construct");
        let return_type = Type::Void;
        let modifiers = node.modifiers.clone();
        let parameters = self.transform_constructor_parameter_list(&node.parameters);

        self.context.class().methods.push(Method { name, return_type, modifiers, parameters, r#abstract: true });
    }

    fn visit_property(&mut self, node: &Property) {
        if !self.context.in_class() {
            return;
        }

        let r#type = node.r#type.as_ref().map(|r| r.get_type()).unwrap_or_else(|| &Type::Mixed).clone();
        let modifiers = node.modifiers.clone();

        for entry in node.entries.iter() {
            let name = entry.variable().stripped;
            let default = entry.is_initialized();

            self.context.class().properties.push(crate::class_like::Property { name, r#type: r#type.clone(), default, modifiers: modifiers.clone() });
        }
    }

    fn visit_variable_property(&mut self, node: &VariableProperty) {
        if !self.context.in_class() {
            return;
        }

        let r#type = node.r#type.as_ref().map(|r| r.get_type()).unwrap_or_else(|| &Type::Mixed).clone();
        let modifiers = PropertyModifierGroup { modifiers: vec![PropertyModifier::Public(Span::default())] };

        for entry in node.entries.iter() {
            let name = entry.variable().stripped;
            let default = entry.is_initialized();

            self.context.class().properties.push(crate::class_like::Property { name, r#type: r#type.clone(), default, modifiers: modifiers.clone() });
        }
    }
}