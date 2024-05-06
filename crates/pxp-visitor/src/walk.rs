use paste::paste;
use pxp_ast::{
    arguments::{Argument, ArgumentList},
    classes::{
        AnonymousClassBody, AnonymousClassExpression, ClassBody, ClassExtends, ClassImplements,
        ClassStatement, ClassishMember,
    },
    constant::{ClassishConstant, ConstantEntry, ConstantStatement},
    control_flow::{
        IfStatement, IfStatementBody, IfStatementElse, IfStatementElseBlock, IfStatementElseIf,
        IfStatementElseIfBlock,
    },
    declares::{DeclareBody, DeclareEntry, DeclareStatement},
    enums::{
        BackedEnumCase, BackedEnumMember, BackedEnumStatement, UnitEnumCase, UnitEnumMember,
        UnitEnumStatement,
    },
    functions::{
        AbstractConstructor, AbstractMethod, ArrowFunctionExpression, ClosureExpression,
        ConcreteConstructor, ConcreteMethod, ConstructorParameter, ConstructorParameterList,
        FunctionBody, FunctionParameter, FunctionParameterList, FunctionStatement, MethodBody,
        ReturnType,
    },
    goto::{GotoStatement, LabelStatement},
    identifiers::{DynamicIdentifier, Identifier},
    interfaces::{InterfaceBody, InterfaceExtends, InterfaceStatement},
    loops::{
        BreakStatement, ContinueStatement, DoWhileStatement, ForStatement, ForStatementBody,
        ForStatementIterator, ForeachStatement, ForeachStatementBody, ForeachStatementIterator,
        Level, WhileStatement, WhileStatementBody,
    },
    namespaces::{BracedNamespace, NamespaceStatement, UnbracedNamespace},
    operators::{
        ArithmeticOperationExpression, AssignmentOperationExpression, BitwiseOperationExpression,
        ComparisonOperationExpression, LogicalOperationExpression,
    },
    properties::{Property, PropertyEntry, VariableProperty},
    traits::{TraitBody, TraitStatement, TraitUsage, TraitUsageAdaptation},
    try_block::{CatchBlock, FinallyBlock, TryStatement},
    variables::{BracedVariableVariable, Variable, VariableVariable},
    ArrayExpression, ArrayIndexExpression, ArrayItem, Case, CastExpression, CloneExpression,
    CoalesceExpression, ConcatExpression, ConstantFetchExpression, DefaultMatchArm, DieExpression,
    EchoStatement, EmptyExpression, ErrorSuppressExpression, EvalExpression, ExitExpression,
    Expression, ExpressionKind, ExpressionStatement, ExpressionStringPart, FunctionCallExpression,
    FunctionClosureCreationExpression, GlobalStatement, GroupUseStatement, HeredocExpression,
    IncludeExpression, IncludeOnceExpression, InstanceofExpression, InterpolatedStringExpression,
    IssetExpression, ListEntry, ListExpression, MatchArm, MatchExpression, MethodCallExpression,
    MethodClosureCreationExpression, NewExpression, NullsafeMethodCallExpression,
    NullsafePropertyFetchExpression, ParenthesizedExpression, PrintExpression,
    PropertyFetchExpression, ReferenceExpression, RequireExpression, RequireOnceExpression,
    ReturnStatement, ShellExecExpression, ShortArrayExpression, ShortTernaryExpression, Statement,
    StatementKind, StaticMethodCallExpression, StaticMethodClosureCreationExpression,
    StaticPropertyFetchExpression, StaticStatement, StaticVar, StaticVariableMethodCallExpression,
    StaticVariableMethodClosureCreationExpression, StringPart, SwitchStatement, TernaryExpression,
    ThrowExpression, UnsetExpression, Use, UseStatement, YieldExpression, YieldFromExpression, data_type::DataType,
};
use pxp_type::Type;

use crate::{VisitorMut, Visitor};

pub fn walk_mut<V: VisitorMut + ?Sized>(visitor: &mut V, program: &mut [Statement]) {
    for statement in program.iter_mut() {
        visitor.visit_statement(statement);
    }
}

pub fn walk_statement_mut<V: VisitorMut + ?Sized>(visitor: &mut V, statement: &mut Statement) {
    match &mut statement.kind {
        StatementKind::FullOpeningTag(stmt) => visitor.visit_full_opening_tag(stmt),
        StatementKind::ShortOpeningTag(stmt) => visitor.visit_short_opening_tag(stmt),
        StatementKind::EchoOpeningTag(stmt) => visitor.visit_echo_opening_tag(stmt),
        StatementKind::ClosingTag(stmt) => visitor.visit_closing_tag(stmt),
        StatementKind::InlineHtml(stmt) => visitor.visit_inline_html(stmt),
        StatementKind::Label(stmt) => visitor.visit_label(stmt),
        StatementKind::Goto(stmt) => visitor.visit_goto(stmt),
        StatementKind::HaltCompiler(stmt) => visitor.visit_halt_compiler(stmt),
        StatementKind::Static(stmt) => visitor.visit_static(stmt),
        StatementKind::DoWhile(stmt) => visitor.visit_do_while(stmt),
        StatementKind::While(stmt) => visitor.visit_while(stmt),
        StatementKind::For(stmt) => visitor.visit_for(stmt),
        StatementKind::Foreach(stmt) => visitor.visit_foreach(stmt),
        StatementKind::Break(stmt) => visitor.visit_break(stmt),
        StatementKind::Continue(stmt) => visitor.visit_continue(stmt),
        StatementKind::Constant(stmt) => visitor.visit_constant(stmt),
        StatementKind::Function(stmt) => visitor.visit_function(stmt),
        StatementKind::Class(stmt) => visitor.visit_class(stmt),
        StatementKind::Trait(stmt) => visitor.visit_trait(stmt),
        StatementKind::Interface(stmt) => visitor.visit_interface(stmt),
        StatementKind::If(stmt) => visitor.visit_if(stmt),
        StatementKind::Switch(stmt) => visitor.visit_switch(stmt),
        StatementKind::Echo(stmt) => visitor.visit_echo(stmt),
        StatementKind::Expression(stmt) => visitor.visit_expression_stmt(stmt),
        StatementKind::Return(stmt) => visitor.visit_return(stmt),
        StatementKind::Namespace(stmt) => visitor.visit_namespace(stmt),
        StatementKind::Use(stmt) => visitor.visit_use(stmt),
        StatementKind::GroupUse(stmt) => visitor.visit_group_use(stmt),
        StatementKind::Comment(stmt) => visitor.visit_comment_stmt(stmt),
        StatementKind::Try(stmt) => visitor.visit_try(stmt),
        StatementKind::UnitEnum(stmt) => visitor.visit_unit_enum(stmt),
        StatementKind::BackedEnum(stmt) => visitor.visit_backed_enum(stmt),
        StatementKind::Block(stmt) => visitor.visit_block(stmt),
        StatementKind::Global(stmt) => visitor.visit_global(stmt),
        StatementKind::Declare(stmt) => visitor.visit_declare(stmt),
        StatementKind::Noop(span) => visitor.visit_noop(*span),
    };
}

pub fn walk_expression_mut<V: VisitorMut + ?Sized>(visitor: &mut V, expression: &mut Expression) {
    match &mut expression.kind {
        ExpressionKind::Missing => visitor.visit_missing_expr(),
        ExpressionKind::Eval(expr) => visitor.visit_eval(expr),
        ExpressionKind::Empty(expr) => visitor.visit_empty(expr),
        ExpressionKind::Die(expr) => visitor.visit_die(expr),
        ExpressionKind::Exit(expr) => visitor.visit_exit(expr),
        ExpressionKind::Isset(expr) => visitor.visit_isset(expr),
        ExpressionKind::Unset(expr) => visitor.visit_unset(expr),
        ExpressionKind::Print(expr) => visitor.visit_print(expr),
        ExpressionKind::Literal(expr) => visitor.visit_literal(expr),
        ExpressionKind::ArithmeticOperation(expr) => visitor.visit_arithmetic_operation(expr),
        ExpressionKind::AssignmentOperation(expr) => visitor.visit_assignment_operation(expr),
        ExpressionKind::BitwiseOperation(expr) => visitor.visit_bitwise_operation(expr),
        ExpressionKind::ComparisonOperation(expr) => visitor.visit_comparison_operation(expr),
        ExpressionKind::LogicalOperation(expr) => visitor.visit_logical_operation(expr),
        ExpressionKind::Concat(expr) => visitor.visit_concat(expr),
        ExpressionKind::Instanceof(expr) => visitor.visit_instanceof(expr),
        ExpressionKind::Reference(expr) => visitor.visit_reference(expr),
        ExpressionKind::Parenthesized(expr) => visitor.visit_parenthesized(expr),
        ExpressionKind::ErrorSuppress(expr) => visitor.visit_error_suppress(expr),
        ExpressionKind::Identifier(expr) => visitor.visit_identifier(expr),
        ExpressionKind::Variable(expr) => visitor.visit_variable(expr),
        ExpressionKind::Include(expr) => visitor.visit_include(expr),
        ExpressionKind::IncludeOnce(expr) => visitor.visit_include_once(expr),
        ExpressionKind::Require(expr) => visitor.visit_require(expr),
        ExpressionKind::RequireOnce(expr) => visitor.visit_require_once(expr),
        ExpressionKind::FunctionCall(expr) => visitor.visit_function_call(expr),
        ExpressionKind::FunctionClosureCreation(expr) => {
            visitor.visit_function_closure_creation(expr)
        }
        ExpressionKind::MethodCall(expr) => visitor.visit_method_call(expr),
        ExpressionKind::MethodClosureCreation(expr) => visitor.visit_method_closure_creation(expr),
        ExpressionKind::NullsafeMethodCall(expr) => visitor.visit_nullsafe_method_call(expr),
        ExpressionKind::StaticMethodCall(expr) => visitor.visit_static_method_call(expr),
        ExpressionKind::StaticVariableMethodCall(expr) => {
            visitor.visit_static_variable_method_call(expr)
        }
        ExpressionKind::StaticMethodClosureCreation(expr) => {
            visitor.visit_static_method_closure_creation(expr)
        }
        ExpressionKind::StaticVariableMethodClosureCreation(expr) => {
            visitor.visit_static_variable_method_closure_creation(expr)
        }
        ExpressionKind::PropertyFetch(expr) => visitor.visit_property_fetch(expr),
        ExpressionKind::NullsafePropertyFetch(expr) => visitor.visit_nullsafe_property_fetch(expr),
        ExpressionKind::StaticPropertyFetch(expr) => visitor.visit_static_property_fetch(expr),
        ExpressionKind::ConstantFetch(expr) => visitor.visit_constant_fetch(expr),
        ExpressionKind::Static => visitor.visit_static_expr(),
        ExpressionKind::Self_ => visitor.visit_self_expr(),
        ExpressionKind::Parent => visitor.visit_parent_expr(),
        ExpressionKind::ShortArray(expr) => visitor.visit_short_array(expr),
        ExpressionKind::Array(expr) => visitor.visit_array(expr),
        ExpressionKind::List(expr) => visitor.visit_list(expr),
        ExpressionKind::Closure(expr) => visitor.visit_closure(expr),
        ExpressionKind::ArrowFunction(expr) => visitor.visit_arrow_function(expr),
        ExpressionKind::New(expr) => visitor.visit_new(expr),
        ExpressionKind::InterpolatedString(expr) => visitor.visit_interpolated_string(expr),
        ExpressionKind::Heredoc(expr) => visitor.visit_heredoc(expr),
        ExpressionKind::Nowdoc(expr) => visitor.visit_nowdoc(expr),
        ExpressionKind::ShellExec(expr) => visitor.visit_shell_exec(expr),
        ExpressionKind::AnonymousClass(expr) => visitor.visit_anonymous_class(expr),
        ExpressionKind::Bool(expr) => visitor.visit_bool(expr),
        ExpressionKind::ArrayIndex(expr) => visitor.visit_array_index(expr),
        ExpressionKind::Null => visitor.visit_null_expr(),
        ExpressionKind::MagicConstant(expr) => visitor.visit_magic_constant(expr),
        ExpressionKind::ShortTernary(expr) => visitor.visit_short_ternary(expr),
        ExpressionKind::Ternary(expr) => visitor.visit_ternary(expr),
        ExpressionKind::Coalesce(expr) => visitor.visit_coalesce(expr),
        ExpressionKind::Clone(expr) => visitor.visit_clone(expr),
        ExpressionKind::Match(expr) => visitor.visit_match(expr),
        ExpressionKind::Throw(expr) => visitor.visit_throw(expr),
        ExpressionKind::Yield(expr) => visitor.visit_yield(expr),
        ExpressionKind::YieldFrom(expr) => visitor.visit_yield_from(expr),
        ExpressionKind::Cast(expr) => visitor.visit_cast(expr),
        ExpressionKind::Noop => visitor.visit_noop_expr(),
    }
}

pub fn walk<V: Visitor + ?Sized>(visitor: &mut V, program: &[Statement]) {
    for statement in program.iter() {
        visitor.visit_statement(statement);
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(visitor: &mut V, statement: &Statement) {
    match &statement.kind {
        StatementKind::FullOpeningTag(stmt) => visitor.visit_full_opening_tag(stmt),
        StatementKind::ShortOpeningTag(stmt) => visitor.visit_short_opening_tag(stmt),
        StatementKind::EchoOpeningTag(stmt) => visitor.visit_echo_opening_tag(stmt),
        StatementKind::ClosingTag(stmt) => visitor.visit_closing_tag(stmt),
        StatementKind::InlineHtml(stmt) => visitor.visit_inline_html(stmt),
        StatementKind::Label(stmt) => visitor.visit_label(stmt),
        StatementKind::Goto(stmt) => visitor.visit_goto(stmt),
        StatementKind::HaltCompiler(stmt) => visitor.visit_halt_compiler(stmt),
        StatementKind::Static(stmt) => visitor.visit_static(stmt),
        StatementKind::DoWhile(stmt) => visitor.visit_do_while(stmt),
        StatementKind::While(stmt) => visitor.visit_while(stmt),
        StatementKind::For(stmt) => visitor.visit_for(stmt),
        StatementKind::Foreach(stmt) => visitor.visit_foreach(stmt),
        StatementKind::Break(stmt) => visitor.visit_break(stmt),
        StatementKind::Continue(stmt) => visitor.visit_continue(stmt),
        StatementKind::Constant(stmt) => visitor.visit_constant(stmt),
        StatementKind::Function(stmt) => visitor.visit_function(stmt),
        StatementKind::Class(stmt) => visitor.visit_class(stmt),
        StatementKind::Trait(stmt) => visitor.visit_trait(stmt),
        StatementKind::Interface(stmt) => visitor.visit_interface(stmt),
        StatementKind::If(stmt) => visitor.visit_if(stmt),
        StatementKind::Switch(stmt) => visitor.visit_switch(stmt),
        StatementKind::Echo(stmt) => visitor.visit_echo(stmt),
        StatementKind::Expression(stmt) => visitor.visit_expression_stmt(stmt),
        StatementKind::Return(stmt) => visitor.visit_return(stmt),
        StatementKind::Namespace(stmt) => visitor.visit_namespace(stmt),
        StatementKind::Use(stmt) => visitor.visit_use(stmt),
        StatementKind::GroupUse(stmt) => visitor.visit_group_use(stmt),
        StatementKind::Comment(stmt) => visitor.visit_comment_stmt(stmt),
        StatementKind::Try(stmt) => visitor.visit_try(stmt),
        StatementKind::UnitEnum(stmt) => visitor.visit_unit_enum(stmt),
        StatementKind::BackedEnum(stmt) => visitor.visit_backed_enum(stmt),
        StatementKind::Block(stmt) => visitor.visit_block(stmt),
        StatementKind::Global(stmt) => visitor.visit_global(stmt),
        StatementKind::Declare(stmt) => visitor.visit_declare(stmt),
        StatementKind::Noop(span) => visitor.visit_noop(*span),
    };
}

pub fn walk_expression<V: Visitor + ?Sized>(visitor: &mut V, expression: &Expression) {
    match &expression.kind {
        ExpressionKind::Missing => visitor.visit_missing_expr(),
        ExpressionKind::Eval(expr) => visitor.visit_eval(expr),
        ExpressionKind::Empty(expr) => visitor.visit_empty(expr),
        ExpressionKind::Die(expr) => visitor.visit_die(expr),
        ExpressionKind::Exit(expr) => visitor.visit_exit(expr),
        ExpressionKind::Isset(expr) => visitor.visit_isset(expr),
        ExpressionKind::Unset(expr) => visitor.visit_unset(expr),
        ExpressionKind::Print(expr) => visitor.visit_print(expr),
        ExpressionKind::Literal(expr) => visitor.visit_literal(expr),
        ExpressionKind::ArithmeticOperation(expr) => visitor.visit_arithmetic_operation(expr),
        ExpressionKind::AssignmentOperation(expr) => visitor.visit_assignment_operation(expr),
        ExpressionKind::BitwiseOperation(expr) => visitor.visit_bitwise_operation(expr),
        ExpressionKind::ComparisonOperation(expr) => visitor.visit_comparison_operation(expr),
        ExpressionKind::LogicalOperation(expr) => visitor.visit_logical_operation(expr),
        ExpressionKind::Concat(expr) => visitor.visit_concat(expr),
        ExpressionKind::Instanceof(expr) => visitor.visit_instanceof(expr),
        ExpressionKind::Reference(expr) => visitor.visit_reference(expr),
        ExpressionKind::Parenthesized(expr) => visitor.visit_parenthesized(expr),
        ExpressionKind::ErrorSuppress(expr) => visitor.visit_error_suppress(expr),
        ExpressionKind::Identifier(expr) => visitor.visit_identifier(expr),
        ExpressionKind::Variable(expr) => visitor.visit_variable(expr),
        ExpressionKind::Include(expr) => visitor.visit_include(expr),
        ExpressionKind::IncludeOnce(expr) => visitor.visit_include_once(expr),
        ExpressionKind::Require(expr) => visitor.visit_require(expr),
        ExpressionKind::RequireOnce(expr) => visitor.visit_require_once(expr),
        ExpressionKind::FunctionCall(expr) => visitor.visit_function_call(expr),
        ExpressionKind::FunctionClosureCreation(expr) => {
            visitor.visit_function_closure_creation(expr)
        }
        ExpressionKind::MethodCall(expr) => visitor.visit_method_call(expr),
        ExpressionKind::MethodClosureCreation(expr) => visitor.visit_method_closure_creation(expr),
        ExpressionKind::NullsafeMethodCall(expr) => visitor.visit_nullsafe_method_call(expr),
        ExpressionKind::StaticMethodCall(expr) => visitor.visit_static_method_call(expr),
        ExpressionKind::StaticVariableMethodCall(expr) => {
            visitor.visit_static_variable_method_call(expr)
        }
        ExpressionKind::StaticMethodClosureCreation(expr) => {
            visitor.visit_static_method_closure_creation(expr)
        }
        ExpressionKind::StaticVariableMethodClosureCreation(expr) => {
            visitor.visit_static_variable_method_closure_creation(expr)
        }
        ExpressionKind::PropertyFetch(expr) => visitor.visit_property_fetch(expr),
        ExpressionKind::NullsafePropertyFetch(expr) => visitor.visit_nullsafe_property_fetch(expr),
        ExpressionKind::StaticPropertyFetch(expr) => visitor.visit_static_property_fetch(expr),
        ExpressionKind::ConstantFetch(expr) => visitor.visit_constant_fetch(expr),
        ExpressionKind::Static => visitor.visit_static_expr(),
        ExpressionKind::Self_ => visitor.visit_self_expr(),
        ExpressionKind::Parent => visitor.visit_parent_expr(),
        ExpressionKind::ShortArray(expr) => visitor.visit_short_array(expr),
        ExpressionKind::Array(expr) => visitor.visit_array(expr),
        ExpressionKind::List(expr) => visitor.visit_list(expr),
        ExpressionKind::Closure(expr) => visitor.visit_closure(expr),
        ExpressionKind::ArrowFunction(expr) => visitor.visit_arrow_function(expr),
        ExpressionKind::New(expr) => visitor.visit_new(expr),
        ExpressionKind::InterpolatedString(expr) => visitor.visit_interpolated_string(expr),
        ExpressionKind::Heredoc(expr) => visitor.visit_heredoc(expr),
        ExpressionKind::Nowdoc(expr) => visitor.visit_nowdoc(expr),
        ExpressionKind::ShellExec(expr) => visitor.visit_shell_exec(expr),
        ExpressionKind::AnonymousClass(expr) => visitor.visit_anonymous_class(expr),
        ExpressionKind::Bool(expr) => visitor.visit_bool(expr),
        ExpressionKind::ArrayIndex(expr) => visitor.visit_array_index(expr),
        ExpressionKind::Null => visitor.visit_null_expr(),
        ExpressionKind::MagicConstant(expr) => visitor.visit_magic_constant(expr),
        ExpressionKind::ShortTernary(expr) => visitor.visit_short_ternary(expr),
        ExpressionKind::Ternary(expr) => visitor.visit_ternary(expr),
        ExpressionKind::Coalesce(expr) => visitor.visit_coalesce(expr),
        ExpressionKind::Clone(expr) => visitor.visit_clone(expr),
        ExpressionKind::Match(expr) => visitor.visit_match(expr),
        ExpressionKind::Throw(expr) => visitor.visit_throw(expr),
        ExpressionKind::Yield(expr) => visitor.visit_yield(expr),
        ExpressionKind::YieldFrom(expr) => visitor.visit_yield_from(expr),
        ExpressionKind::Cast(expr) => visitor.visit_cast(expr),
        ExpressionKind::Noop => visitor.visit_noop_expr(),
    }
}

macro_rules! walk_mut {
    (
        using($v:ident, $n:ident);

        $($label:ident: $node:ty => $body:block )+
    ) => {
        $(
            paste! {
                pub fn [<$label _mut>]<V: VisitorMut + ?Sized>($v: &mut V, $n: &mut $node) $body
            }
        )+
    }
}

macro_rules! walk {
    (
        using($v:ident, $n:ident);

        $($label:ident: $node:ty => $body:block )+
    ) => {
        $(
            pub fn $label<V: Visitor + ?Sized>($v: &mut V, $n: &$node) $body
        )+
    }
}

walk_mut! {
    using(visitor, node);

    walk_label: LabelStatement => {
        visitor.visit_simple_identifier(&mut node.label)
    }

    walk_goto: GotoStatement => {
        visitor.visit_simple_identifier(&mut node.label)
    }

    walk_static: StaticStatement => {
        for variable in node.vars.iter_mut() {
            visitor.visit_static_var(variable)
        }
    }

    walk_static_var: StaticVar => {
        visitor.visit_variable(&mut node.var);

        if let Some(default) = &mut node.default {
            visitor.visit_expression(default);
        }
    }

    walk_global: GlobalStatement => {
        for variable in node.variables.iter_mut() {
            visitor.visit_variable(variable)
        }
    }

    walk_do_while: DoWhileStatement => {
        visitor.visit_statement(&mut node.body);
        visitor.visit_expression(&mut node.condition);
    }

    walk_while: WhileStatement => {
        visitor.visit_expression(&mut node.condition);
        visitor.visit_while_statement_body(&mut node.body);
    }

    walk_while_statement_body: WhileStatementBody => {
        match node {
            WhileStatementBody::Statement { statement } => {
                visitor.visit_statement(statement.as_mut());
            },
            WhileStatementBody::Block { statements, .. } => {
                visitor.visit(statements)
            }
        }
    }

    walk_for: ForStatement => {
        visitor.visit_for_statement_iterator(&mut node.iterator);
        visitor.visit_for_statement_body(&mut node.body);
    }

    walk_for_statement_iterator: ForStatementIterator => {
        for init in node.initializations.iter_mut() {
            visitor.visit_expression(init);
        }

        for condition in node.conditions.iter_mut() {
            visitor.visit_expression(condition);
        }

        for r#loop in node.r#loop.iter_mut() {
            visitor.visit_expression(r#loop);
        }
    }

    walk_for_statement_body: ForStatementBody => {
        match node {
            ForStatementBody::Statement { statement } => {
                visitor.visit_statement(statement.as_mut());
            },
            ForStatementBody::Block { statements, .. } => {
                visitor.visit(statements)
            }
        }
    }

    walk_foreach: ForeachStatement => {
        visitor.visit_foreach_statement_iterator(&mut node.iterator);
        visitor.visit_foreach_statement_body(&mut node.body);
    }

    walk_foreach_statement_iterator: ForeachStatementIterator => {
        match node {
            ForeachStatementIterator::Value { expression, value, .. } => {
                visitor.visit_expression(expression);
                visitor.visit_expression(value);
            },
            ForeachStatementIterator::KeyAndValue { expression, key, value, .. } => {
                visitor.visit_expression(expression);
                visitor.visit_expression(key);
                visitor.visit_expression(value);
            },
        }
    }

    walk_foreach_statement_body: ForeachStatementBody => {
        match node {
            ForeachStatementBody::Statement { statement } => {
                visitor.visit_statement(statement)
            },
            ForeachStatementBody::Block { statements, .. } => {
                visitor.visit(statements)
            }
        }
    }

    walk_if: IfStatement => {
        visitor.visit_expression(&mut node.condition);
        visitor.visit_if_statement_body(&mut node.body);
    }

    walk_if_statement_body: IfStatementBody => {
        match node {
            IfStatementBody::Statement { statement, elseifs, r#else } => {
                visitor.visit_statement(statement);

                for r#elseif in elseifs.iter_mut() {
                    visitor.visit_if_statement_elseif(r#elseif);
                }

                if let Some(r#else) = r#else {
                    visitor.visit_if_statement_else(r#else);
                }
            },
            IfStatementBody::Block { statements, elseifs, r#else, .. } => {
                visitor.visit(statements);

                for r#elseif in elseifs.iter_mut() {
                    visitor.visit_if_statement_elseif_block(r#elseif);
                }

                if let Some(r#else) = r#else {
                    visitor.visit_if_statement_else_block(r#else);
                }
            },
        }
    }

    walk_if_statement_elseif: IfStatementElseIf => {
        visitor.visit_expression(&mut node.condition);
        visitor.visit_statement(&mut node.statement);
    }

    walk_if_statement_elseif_block: IfStatementElseIfBlock => {
        visitor.visit_expression(&mut node.condition);
        visitor.visit(&mut node.statements);
    }

    walk_if_statement_else: IfStatementElse => {
        visitor.visit_statement(&mut node.statement);
    }

    walk_if_statement_else_block: IfStatementElseBlock => {
        visitor.visit(&mut node.statements);
    }

    walk_switch: SwitchStatement => {
        visitor.visit_expression(&mut node.condition);

        for case in node.cases.iter_mut() {
            visitor.visit_switch_case(case);
        }
    }

    walk_switch_case: Case => {
        if let Some(condition) = &mut node.condition {
            visitor.visit_expression(condition);
        }

        visitor.visit(&mut node.body);
    }

    walk_level: Level => {
        match node {
            Level::Literal(literal) => visitor.visit_literal(literal),
            Level::Parenthesized { level, .. } => visitor.visit_level(level),
        }
    }

    walk_break: BreakStatement => {
        if let Some(level) = &mut node.level {
            visitor.visit_level(level);
        }
    }

    walk_continue: ContinueStatement => {
        if let Some(level) = &mut node.level {
            visitor.visit_level(level);
        }
    }

    walk_constant: ConstantStatement => {
        for entry in node.entries.iter_mut() {
            visitor.visit_constant_entry(entry);
        }
    }

    walk_constant_entry: ConstantEntry => {
        visitor.visit_simple_identifier(&mut node.name);
        visitor.visit_expression(&mut node.value);
    }

    walk_function: FunctionStatement => {
        // FIXME: Walk attributes here.
        visitor.visit_simple_identifier(&mut node.name);
        visitor.visit_function_parameter_list(&mut node.parameters);
        visitor.visit_function_body(&mut node.body);
    }

    walk_function_parameter_list: FunctionParameterList => {
        for parameter in node.parameters.iter_mut() {
            visitor.visit_function_parameter(parameter);
        }
    }

    walk_function_parameter: FunctionParameter => {
        visitor.visit_simple_variable(&mut node.name);
        // FIXME: Walk attributes here.

        if let Some(ty) = &mut node.data_type {
            visitor.visit_data_type(ty);
        }

        if let Some(default) = &mut node.default {
            visitor.visit_expression(default);
        }
    }

    walk_function_body: FunctionBody => {
        visitor.visit(&mut node.statements);
    }

    walk_class: ClassStatement => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.
        visitor.visit_simple_identifier(&mut node.name);

        if let Some(extends) = &mut node.extends {
            visitor.visit_class_extends(extends);
        }

        if let Some(implements) = &mut node.implements {
            visitor.visit_class_implements(implements);
        }

        visitor.visit_class_body(&mut node.body);
    }

    walk_class_extends: ClassExtends => {
        visitor.visit_simple_identifier(&mut node.parent);
    }

    walk_class_implements: ClassImplements => {
        for interface in node.interfaces.iter_mut() {
            visitor.visit_simple_identifier(interface);
        }
    }

    walk_class_body: ClassBody => {
        for member in node.members.iter_mut() {
            visitor.visit_classish_member(member);
        }
    }

    walk_classish_member: ClassishMember => {
        match node {
            ClassishMember::Constant(constant) => {
                visitor.visit_classish_constant(constant);
            },
            ClassishMember::TraitUsage(usage) => {
                visitor.visit_trait_usage(usage);
            },
            ClassishMember::Property(property) => {
                visitor.visit_property(property);
            },
            ClassishMember::VariableProperty(property) => {
                visitor.visit_variable_property(property);
            },
            ClassishMember::AbstractMethod(method) => {
                visitor.visit_abstract_method(method);
            },
            ClassishMember::AbstractConstructor(method) => {
                visitor.visit_abstract_constructor(method);
            },
            ClassishMember::ConcreteMethod(method) => {
                visitor.visit_concrete_method(method);
            },
            ClassishMember::ConcreteConstructor(method) => {
                visitor.visit_concrete_constructor(method);
            },
        }
    }

    walk_classish_constant: ClassishConstant => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        for entries in node.entries.iter_mut() {
            visitor.visit_constant_entry(entries);
        }
    }

    walk_trait_usage: TraitUsage => {
        for r#trait in node.traits.iter_mut() {
            visitor.visit_simple_identifier(r#trait);
        }

        for adaptation in node.adaptations.iter_mut() {
            visitor.visit_trait_usage_adaptation(adaptation);
        }
    }

    walk_trait_usage_adaptation: TraitUsageAdaptation => {
        match node {
            TraitUsageAdaptation::Alias { r#trait, method, alias, visibility } => {
                if let Some(r#trait) = r#trait {
                    visitor.visit_simple_identifier(r#trait);
                }

                visitor.visit_simple_identifier(method);
                visitor.visit_simple_identifier(alias);

                if let Some(_visibility) = visibility {
                    // FIXME: Visit visibility here.
                    // visitor.visit_visibility_modifier(visibility);
                }
            },
            TraitUsageAdaptation::Visibility { r#trait, method, visibility: _visibility } => {
                if let Some(r#trait) = r#trait {
                    visitor.visit_simple_identifier(r#trait);
                }

                visitor.visit_simple_identifier(method);
                // FIXME: Visit visibility here.
            },
            TraitUsageAdaptation::Precedence { r#trait, method, insteadof } => {
                if let Some(r#trait) = r#trait {
                    visitor.visit_simple_identifier(r#trait);
                }

                visitor.visit_simple_identifier(method);

                for insteadof in insteadof.iter_mut() {
                    visitor.visit_simple_identifier(insteadof);
                }
            }
        }
    }

    walk_property: Property => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        if let Some(ty) = &mut node.r#type {
            visitor.visit_data_type(ty);
        }

        for entry in node.entries.iter_mut() {
            visitor.visit_property_entry(entry);
        }
    }

    walk_property_entry: PropertyEntry => {
        match node {
            PropertyEntry::Uninitialized { variable } => {
                visitor.visit_simple_variable(variable);
            },
            PropertyEntry::Initialized { variable, value, .. } => {
                visitor.visit_simple_variable(variable);
                visitor.visit_expression(value);
            },
        }
    }

    walk_variable_property: VariableProperty => {
        // FIXME: Walk attributes here.
        // FIXME: Walk type here.

        for entry in node.entries.iter_mut() {
            visitor.visit_property_entry(entry);
        }
    }

    walk_abstract_method: AbstractMethod => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_simple_identifier(&mut node.name);
        visitor.visit_function_parameter_list(&mut node.parameters);

        if let Some(ty) = &mut node.return_type {
            visitor.visit_return_type(ty);
        }
    }

    walk_abstract_constructor: AbstractConstructor => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_constructor_parameter_list(&mut node.parameters);
    }

    walk_constructor_parameter_list: ConstructorParameterList => {
        for parameter in node.parameters.iter_mut() {
            visitor.visit_constructor_parameter(parameter);
        }
    }

    walk_constructor_parameter: ConstructorParameter => {
        // FIXME: Walk attributes here.
        // FIXME: Visit modifiers here.
        // FIXME: Visit type here.
        visitor.visit_simple_variable(&mut node.name);

        if let Some(default) = &mut node.default {
            visitor.visit_expression(default);
        }
    }

    walk_concrete_method: ConcreteMethod => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_simple_identifier(&mut node.name);
        visitor.visit_function_parameter_list(&mut node.parameters);

        if let Some(ty) = &mut node.return_type {
            visitor.visit_return_type(ty);
        }

        visitor.visit_method_body(&mut node.body);
    }

    walk_method_body: MethodBody => {
        visitor.visit(&mut node.statements);
    }

    walk_concrete_constructor: ConcreteConstructor => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_constructor_parameter_list(&mut node.parameters);
        visitor.visit_method_body(&mut node.body);
    }

    walk_interface: InterfaceStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&mut node.name);

        if let Some(extends) = &mut node.extends {
            visitor.visit_interface_extends(extends);
        }

        visitor.visit_interface_body(&mut node.body);
    }

    walk_interface_extends: InterfaceExtends => {
        for parent in node.parents.iter_mut() {
            visitor.visit_simple_identifier(parent);
        }
    }

    walk_interface_body: InterfaceBody => {
        for member in node.members.iter_mut() {
            visitor.visit_classish_member(member);
        }
    }

    walk_trait: TraitStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&mut node.name);
        visitor.visit_trait_body(&mut node.body);
    }

    walk_trait_body: TraitBody => {
        for member in node.members.iter_mut() {
            visitor.visit_classish_member(member);
        }
    }

    walk_echo: EchoStatement => {
        for value in node.values.iter_mut() {
            visitor.visit_expression(value);
        }
    }

    walk_expression_stmt: ExpressionStatement => {
        visitor.visit_expression(&mut node.expression);
    }

    walk_return: ReturnStatement => {
        if let Some(expression) = &mut node.value {
            visitor.visit_expression(expression);
        }
    }

    walk_namespace: NamespaceStatement => {
        match node {
            NamespaceStatement::Unbraced(node) => {
                visitor.visit_unbraced_namespace(node);
            },
            NamespaceStatement::Braced(node) => {
                visitor.visit_braced_namespace(node);
            },
        }
    }

    walk_unbraced_namespace: UnbracedNamespace => {
        visitor.visit_simple_identifier(&mut node.name);
        visitor.visit(&mut node.statements);
    }

    walk_braced_namespace: BracedNamespace => {
        if let Some(name) = &mut node.name {
            visitor.visit_simple_identifier(name);
        }

        visitor.visit(&mut node.body.statements);
    }

    walk_use: UseStatement => {
        for r#use in node.uses.iter_mut() {
            visitor.visit_use_use(r#use);
        }
    }

    walk_use_use: Use => {
        visitor.visit_simple_identifier(&mut node.name);

        if let Some(alias) = &mut node.alias {
            visitor.visit_simple_identifier(alias);
        }
    }

    walk_group_use: GroupUseStatement => {
        visitor.visit_simple_identifier(&mut node.prefix);

        for r#use in node.uses.iter_mut() {
            visitor.visit_use_use(r#use);
        }
    }

    walk_try: TryStatement => {
        visitor.visit(&mut node.body);

        for catch in node.catches.iter_mut() {
            visitor.visit_catch_block(catch);
        }

        if let Some(finally) = &mut node.finally {
            visitor.visit_finally_block(finally);
        }
    }

    walk_catch_block: CatchBlock => {
        if let Some(variable) = &mut node.var {
            visitor.visit_simple_variable(variable);
        }

        visitor.visit(&mut node.body);
    }

    walk_finally_block: FinallyBlock => {
        visitor.visit(&mut node.body);
    }

    walk_unit_enum: UnitEnumStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&mut node.name);

        for implements in node.implements.iter_mut() {
            visitor.visit_simple_identifier(implements);
        }

        for member in node.body.members.iter_mut() {
            visitor.visit_unit_enum_member(member);
        }
    }

    walk_unit_enum_member: UnitEnumMember => {
        match node {
            UnitEnumMember::Case(node) => {
                visitor.visit_unit_enum_case(node);
            }
            UnitEnumMember::Classish(node) => {
                visitor.visit_classish_member(node);
            }
        }
    }

    walk_unit_enum_case: UnitEnumCase => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&mut node.name);
    }

    walk_backed_enum: BackedEnumStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&mut node.name);

        for implements in node.implements.iter_mut() {
            visitor.visit_simple_identifier(implements);
        }

        for member in node.body.members.iter_mut() {
            visitor.visit_backed_enum_member(member);
        }
    }

    walk_backed_enum_member: BackedEnumMember => {
        match node {
            BackedEnumMember::Case(node) => {
                visitor.visit_backed_enum_case(node);
            }
            BackedEnumMember::Classish(node) => {
                visitor.visit_classish_member(node);
            }
        }
    }

    walk_backed_enum_case: BackedEnumCase => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&mut node.name);
        visitor.visit_expression(&mut node.value);
    }

    walk_declare: DeclareStatement => {
        for entry in node.entries.entries.iter_mut() {
            visitor.visit_declare_entry(entry);
        }

        visitor.visit_declare_body(&mut node.body);
    }

    walk_declare_entry: DeclareEntry => {
        visitor.visit_simple_identifier(&mut node.key);
        visitor.visit_literal(&mut node.value);
    }

    walk_declare_body: DeclareBody => {
        match node {
            DeclareBody::Noop { .. } => {},
            DeclareBody::Braced { statements, .. } => {
                visitor.visit(statements);
            },
            DeclareBody::Expression { expression, .. } => {
                visitor.visit_expression(expression);
            },
            DeclareBody::Block { statements, .. } => {
                visitor.visit(statements);
            },
        }
    }

    walk_argument_list: ArgumentList => {
        for argument in node.arguments.iter_mut() {
            visitor.visit_argument(argument);
        }
    }

    walk_argument: Argument => {
        match node {
            Argument::Positional(node) => {
                visitor.visit_expression(&mut node.value);
            },
            Argument::Named(node) => {
                visitor.visit_simple_identifier(&mut node.name);
                visitor.visit_expression(&mut node.value);
            },
        }
    }

    walk_eval: EvalExpression => {
        if let Some(argument) = &mut node.argument.argument {
            visitor.visit_argument(argument);
        }
    }

    walk_empty: EmptyExpression => {
        if let Some(argument) = &mut node.argument.argument {
            visitor.visit_argument(argument);
        }
    }

    walk_die: DieExpression => {
        if let Some(argument) = &mut node.argument {
            if let Some(argument) = &mut argument.argument {
                visitor.visit_argument(argument);
            }
        }
    }

    walk_exit: ExitExpression => {
        if let Some(argument) = &mut node.argument {
            if let Some(argument) = &mut argument.argument {
                visitor.visit_argument(argument);
            }
        }
    }

    walk_isset: IssetExpression => {
        visitor.visit_argument_list(&mut node.arguments);
    }

    walk_unset: UnsetExpression => {
        visitor.visit_argument_list(&mut node.arguments);
    }

    walk_print: PrintExpression => {
        if let Some(value) = &mut node.value {
            visitor.visit_expression(value);
        }

        if let Some(argument) = &mut node.argument {
            if let Some(argument) = &mut argument.argument {
                visitor.visit_argument(argument);
            }
        }
    }

    walk_arithmetic_operation: ArithmeticOperationExpression => {
        match node {
            ArithmeticOperationExpression::Addition {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::Subtraction {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::Multiplication {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::Division {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::Modulo {
                left, right, ..
            }  => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::Exponentiation {
                left, right, ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::Negative {
                right, ..
            } => {
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::Positive {
                right,
                ..
            } => {
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::PreIncrement {
                right, ..
            } => {
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::PostIncrement {
                left, ..
            } => {
                visitor.visit_expression(left);
            },
            ArithmeticOperationExpression::PreDecrement {
                right, ..
            } => {
                visitor.visit_expression(right);
            },
            ArithmeticOperationExpression::PostDecrement {
                left, ..
            } => {
                visitor.visit_expression(left);
            },
        }
    }

    walk_assignment_operation: AssignmentOperationExpression => {
        match node {
            AssignmentOperationExpression::Assign {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Addition {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Subtraction {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Multiplication {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Division {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Modulo {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Exponentiation {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Concat {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::BitwiseAnd {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::BitwiseOr {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::BitwiseXor {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::LeftShift {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::RightShift {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Coalesce {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            }
        }
    }

    walk_bitwise_operation: BitwiseOperationExpression => {
        match node {
            BitwiseOperationExpression::And {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::Or {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::Xor {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::LeftShift {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::RightShift {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            BitwiseOperationExpression::Not {
                right,
                ..
            } => {
                visitor.visit_expression(right);
            },
        }
    }

    walk_comparison_operation: ComparisonOperationExpression => {
        match node {
            ComparisonOperationExpression::Equal {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::Identical {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::NotEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::AngledNotEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::NotIdentical {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::LessThan {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::GreaterThan {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::LessThanOrEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::GreaterThanOrEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::Spaceship {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            }
        }
    }

    walk_logical_operation: LogicalOperationExpression => {
        match node {
            LogicalOperationExpression::And {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::Or {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::LogicalAnd {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::LogicalOr {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::LogicalXor {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(left);
                visitor.visit_expression(right);
            },
            LogicalOperationExpression::Not {
                right,
                ..
            } => {
                visitor.visit_expression(right);
            },
        }
    }

    walk_concat: ConcatExpression => {
        visitor.visit_expression(&mut node.left);
        visitor.visit_expression(&mut node.right);
    }

    walk_instanceof: InstanceofExpression => {
        visitor.visit_expression(&mut node.left);
        visitor.visit_expression(&mut node.right);
    }

    walk_reference: ReferenceExpression => {
        visitor.visit_expression(&mut node.right);
    }

    walk_parenthesized: ParenthesizedExpression => {
        visitor.visit_expression(&mut node.expr);
    }

    walk_error_suppress: ErrorSuppressExpression => {
        visitor.visit_expression(&mut node.expr);
    }

    walk_identifier: Identifier => {
        match node {
            Identifier::SimpleIdentifier(node) => {
                visitor.visit_simple_identifier(node);
            },
            Identifier::DynamicIdentifier(node) => {
                visitor.visit_dynamic_identifier(node);
            },
        }
    }

    walk_dynamic_identifier: DynamicIdentifier => {
        visitor.visit_expression(&mut node.expr);
    }

    walk_variable: Variable => {
        match node {
            Variable::SimpleVariable(node) => {
                visitor.visit_simple_variable(node);
            },
            Variable::VariableVariable(node) => {
                visitor.visit_variable_variable(node);
            },
            Variable::BracedVariableVariable(node) => {
                visitor.visit_braced_variable_variable(node);
            }
        }
    }

    walk_variable_variable: VariableVariable => {
        visitor.visit_variable(&mut node.variable);
    }

    walk_braced_variable_variable: BracedVariableVariable => {
        visitor.visit_expression(&mut node.variable);
    }

    walk_include: IncludeExpression => {
        visitor.visit_expression(&mut node.path);
    }

    walk_include_once: IncludeOnceExpression => {
        visitor.visit_expression(&mut node.path);
    }

    walk_require: RequireExpression => {
        visitor.visit_expression(&mut node.path);
    }

    walk_require_once: RequireOnceExpression => {
        visitor.visit_expression(&mut node.path);
    }

    walk_function_call: FunctionCallExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_argument_list(&mut node.arguments);
    }

    walk_function_closure_creation: FunctionClosureCreationExpression => {
        visitor.visit_expression(&mut node.target);
    }

    walk_method_call: MethodCallExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_expression(&mut node.method);
        visitor.visit_argument_list(&mut node.arguments);
    }

    walk_method_closure_creation: MethodClosureCreationExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_expression(&mut node.method);
    }

    walk_nullsafe_method_call: NullsafeMethodCallExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_expression(&mut node.method);
        visitor.visit_argument_list(&mut node.arguments);
    }

    walk_static_method_call: StaticMethodCallExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_identifier(&mut node.method);
        visitor.visit_argument_list(&mut node.arguments);
    }

    walk_static_variable_method_call: StaticVariableMethodCallExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_variable(&mut node.method);
        visitor.visit_argument_list(&mut node.arguments);
    }

    walk_static_method_closure_creation: StaticMethodClosureCreationExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_identifier(&mut node.method);
    }

    walk_static_variable_method_closure_creation: StaticVariableMethodClosureCreationExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_variable(&mut node.method);
    }

    walk_property_fetch: PropertyFetchExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_expression(&mut node.property);
    }

    walk_nullsafe_property_fetch: NullsafePropertyFetchExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_expression(&mut node.property);
    }

    walk_static_property_fetch: StaticPropertyFetchExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_variable(&mut node.property);
    }

    walk_constant_fetch: ConstantFetchExpression => {
        visitor.visit_expression(&mut node.target);
        visitor.visit_identifier(&mut node.constant);
    }

    walk_array_item: ArrayItem => {
        match node {
            ArrayItem::Skipped => {},
            ArrayItem::Value {
                value
            } => {
                visitor.visit_expression(value);
            },
            ArrayItem::ReferencedValue {
                value, ..
            } => {
                visitor.visit_expression(value);
            },
            ArrayItem::SpreadValue {
                value, ..
            } => {
                visitor.visit_expression(value);
            },
            ArrayItem::KeyValue {
                key,
                value,
                ..
            } => {
                visitor.visit_expression(key);
                visitor.visit_expression(value);
            },
            ArrayItem::ReferencedKeyValue {
                key,
                value, ..
            } => {
                visitor.visit_expression(key);
                visitor.visit_expression(value);
            },
        }
    }

    walk_short_array: ShortArrayExpression => {
        for item in node.items.inner.iter_mut() {
            visitor.visit_array_item(item);
        }
    }

    walk_array: ArrayExpression => {
        for item in node.items.inner.iter_mut() {
            visitor.visit_array_item(item);
        }
    }

    walk_list: ListExpression => {
        for entry in node.items.iter_mut() {
            visitor.visit_list_entry(entry);
        }
    }

    walk_list_entry: ListEntry => {
        match node {
            ListEntry::Skipped => {},
            ListEntry::Value {
                value
            } => {
                visitor.visit_expression(value);
            },
            ListEntry::KeyValue {
                key,
                value,
                ..
            } => {
                visitor.visit_expression(key);
                visitor.visit_expression(value);
            },
        }
    }

    walk_closure: ClosureExpression => {
        // FIXME: Walk attributes here.

        visitor.visit_function_parameter_list(&mut node.parameters);

        if let Some(return_type) = &mut node.return_type {
            visitor.visit_return_type(return_type);
        }

        visitor.visit_function_body(&mut node.body);
    }

    walk_arrow_function: ArrowFunctionExpression => {
        // FIXME: Walk attributes here.

        visitor.visit_function_parameter_list(&mut node.parameters);
        visitor.visit_expression(&mut node.body);
    }

    walk_new: NewExpression => {
        visitor.visit_expression(&mut node.target);

        if let Some(arguments) = &mut node.arguments {
            visitor.visit_argument_list(arguments);
        }
    }

    walk_interpolated_string: InterpolatedStringExpression => {
        for part in node.parts.iter_mut() {
            visitor.visit_string_part(part);
        }
    }

    walk_string_part: StringPart => {
        match node {
            StringPart::Literal(node) => {
                visitor.visit_literal_string_part(node);
            },
            StringPart::Expression(node) => {
                visitor.visit_expression_string_part(node);
            }
        }
    }

    walk_expression_string_part: ExpressionStringPart => {
        visitor.visit_expression(&mut node.expression);
    }

    walk_heredoc: HeredocExpression => {
        for part in node.parts.iter_mut() {
            visitor.visit_string_part(part);
        }
    }

    walk_shell_exec: ShellExecExpression => {
        for part in node.parts.iter_mut() {
            visitor.visit_string_part(part);
        }
    }

    walk_anonymous_class: AnonymousClassExpression => {
        // FIXME: Walk attributes here.

        if let Some(extends) = &mut node.extends {
            visitor.visit_class_extends(extends);
        }

        if let Some(implements) = &mut node.implements {
            visitor.visit_class_implements(implements);
        }

        visitor.visit_anonymous_class_body(&mut node.body);
    }

    walk_anonymous_class_body: AnonymousClassBody => {
        for member in node.members.iter_mut() {
            visitor.visit_classish_member(member);
        }
    }

    walk_array_index: ArrayIndexExpression => {
        visitor.visit_expression(&mut node.array);

        if let Some(index) = &mut node.index {
            visitor.visit_expression(index);
        }
    }

    walk_short_ternary: ShortTernaryExpression => {
        visitor.visit_expression(&mut node.condition);
        visitor.visit_expression(&mut node.r#else);
    }

    walk_ternary: TernaryExpression => {
        visitor.visit_expression(&mut node.condition);
        visitor.visit_expression(&mut node.then);
        visitor.visit_expression(&mut node.r#else);
    }

    walk_coalesce: CoalesceExpression => {
        visitor.visit_expression(&mut node.lhs);
        visitor.visit_expression(&mut node.rhs);
    }

    walk_clone: CloneExpression => {
        visitor.visit_expression(&mut node.target);
    }

    walk_match: MatchExpression => {
        visitor.visit_expression(&mut node.condition);

        for arm in node.arms.iter_mut() {
            visitor.visit_match_arm(arm);
        }

        if let Some(default) = &mut node.default {
            visitor.visit_default_match_arm(default);
        }
    }

    walk_match_arm: MatchArm => {
        for condition in &mut node.conditions.iter_mut() {
            visitor.visit_expression(condition);
        }

        visitor.visit_expression(&mut node.body);
    }

    walk_default_match_arm: DefaultMatchArm => {
        visitor.visit_expression(&mut node.body);
    }

    walk_throw: ThrowExpression => {
        visitor.visit_expression(&mut node.value);
    }

    walk_yield: YieldExpression => {
        if let Some(key) = &mut node.key {
            visitor.visit_expression(key);
        }

        if let Some(value) = &mut node.value {
            visitor.visit_expression(value);
        }
    }

    walk_yield_from: YieldFromExpression => {
        visitor.visit_expression(&mut node.value);
    }

    walk_cast: CastExpression => {
        visitor.visit_expression(&mut node.value);
    }

    walk_return_type: ReturnType => {
        visitor.visit_data_type(&mut node.data_type);
    }

    walk_data_type: DataType => {
        visitor.visit_type(&mut node.kind);
    }

    walk_type: Type => {
        match node {
            Type::Nullable(ty) => {
                visitor.visit_type(ty);
            },
            Type::Union(tys) |
            Type::Intersection(tys) => {
                for ty in tys.iter_mut() {
                    visitor.visit_type(ty);
                }
            },
            _ => {},
        }
    }
}

walk! {
    using(visitor, node);

    walk_label: LabelStatement => {
        visitor.visit_simple_identifier(&node.label)
    }

    walk_goto: GotoStatement => {
        visitor.visit_simple_identifier(&node.label)
    }

    walk_static: StaticStatement => {
        for variable in node.vars.iter() {
            visitor.visit_static_var(variable)
        }
    }

    walk_static_var: StaticVar => {
        visitor.visit_variable(&node.var);

        if let Some(default) = &node.default {
            visitor.visit_expression(default);
        }
    }

    walk_global: GlobalStatement => {
        for variable in node.variables.iter() {
            visitor.visit_variable(variable)
        }
    }

    walk_do_while: DoWhileStatement => {
        visitor.visit_statement(&node.body);
        visitor.visit_expression(&node.condition);
    }

    walk_while: WhileStatement => {
        visitor.visit_expression(&node.condition);
        visitor.visit_while_statement_body(&node.body);
    }

    walk_while_statement_body: WhileStatementBody => {
        match node {
            WhileStatementBody::Statement { statement } => {
                visitor.visit_statement(&statement);
            },
            WhileStatementBody::Block { statements, .. } => {
                visitor.visit(&statements)
            }
        }
    }

    walk_for: ForStatement => {
        visitor.visit_for_statement_iterator(&node.iterator);
        visitor.visit_for_statement_body(&node.body);
    }

    walk_for_statement_iterator: ForStatementIterator => {
        for init in node.initializations.iter() {
            visitor.visit_expression(init);
        }

        for condition in node.conditions.iter() {
            visitor.visit_expression(condition);
        }

        for r#loop in node.r#loop.iter() {
            visitor.visit_expression(r#loop);
        }
    }

    walk_for_statement_body: ForStatementBody => {
        match node {
            ForStatementBody::Statement { statement } => {
                visitor.visit_statement(&statement);
            },
            ForStatementBody::Block { statements, .. } => {
                visitor.visit(&statements)
            }
        }
    }

    walk_foreach: ForeachStatement => {
        visitor.visit_foreach_statement_iterator(&node.iterator);
        visitor.visit_foreach_statement_body(&node.body);
    }

    walk_foreach_statement_iterator: ForeachStatementIterator => {
        match node {
            ForeachStatementIterator::Value { expression, value, .. } => {
                visitor.visit_expression(&expression);
                visitor.visit_expression(&value);
            },
            ForeachStatementIterator::KeyAndValue { expression, key, value, .. } => {
                visitor.visit_expression(&expression);
                visitor.visit_expression(&key);
                visitor.visit_expression(&value);
            },
        }
    }

    walk_foreach_statement_body: ForeachStatementBody => {
        match node {
            ForeachStatementBody::Statement { statement } => {
                visitor.visit_statement(&statement)
            },
            ForeachStatementBody::Block { statements, .. } => {
                visitor.visit(&statements)
            }
        }
    }

    walk_if: IfStatement => {
        visitor.visit_expression(&node.condition);
        visitor.visit_if_statement_body(&node.body);
    }

    walk_if_statement_body: IfStatementBody => {
        match node {
            IfStatementBody::Statement { statement, elseifs, r#else } => {
                visitor.visit_statement(&statement);

                for r#elseif in elseifs.iter() {
                    visitor.visit_if_statement_elseif(r#elseif);
                }

                if let Some(r#else) = r#else {
                    visitor.visit_if_statement_else(&r#else);
                }
            },
            IfStatementBody::Block { statements, elseifs, r#else, .. } => {
                visitor.visit(&statements);

                for r#elseif in elseifs.iter() {
                    visitor.visit_if_statement_elseif_block(r#elseif);
                }

                if let Some(r#else) = r#else {
                    visitor.visit_if_statement_else_block(&r#else);
                }
            },
        }
    }

    walk_if_statement_elseif: IfStatementElseIf => {
        visitor.visit_expression(&node.condition);
        visitor.visit_statement(&node.statement);
    }

    walk_if_statement_elseif_block: IfStatementElseIfBlock => {
        visitor.visit_expression(&node.condition);
        visitor.visit(&node.statements);
    }

    walk_if_statement_else: IfStatementElse => {
        visitor.visit_statement(&node.statement);
    }

    walk_if_statement_else_block: IfStatementElseBlock => {
        visitor.visit(&node.statements);
    }

    walk_switch: SwitchStatement => {
        visitor.visit_expression(&node.condition);

        for case in node.cases.iter() {
            visitor.visit_switch_case(case);
        }
    }

    walk_switch_case: Case => {
        if let Some(condition) = &node.condition {
            visitor.visit_expression(condition);
        }

        visitor.visit(&node.body);
    }

    walk_level: Level => {
        match node {
            Level::Literal(literal) => visitor.visit_literal(&literal),
            Level::Parenthesized { level, .. } => visitor.visit_level(&level),
        }
    }

    walk_break: BreakStatement => {
        if let Some(level) = &node.level {
            visitor.visit_level(level);
        }
    }

    walk_continue: ContinueStatement => {
        if let Some(level) = &node.level {
            visitor.visit_level(level);
        }
    }

    walk_constant: ConstantStatement => {
        for entry in node.entries.iter() {
            visitor.visit_constant_entry(entry);
        }
    }

    walk_constant_entry: ConstantEntry => {
        visitor.visit_simple_identifier(&node.name);
        visitor.visit_expression(&node.value);
    }

    walk_function: FunctionStatement => {
        // FIXME: Walk attributes here.
        visitor.visit_simple_identifier(&node.name);
        visitor.visit_function_parameter_list(&node.parameters);
        visitor.visit_function_body(&node.body);
    }

    walk_function_parameter_list: FunctionParameterList => {
        for parameter in node.parameters.iter() {
            visitor.visit_function_parameter(parameter);
        }
    }

    walk_function_parameter: FunctionParameter => {
        visitor.visit_simple_variable(&node.name);
        // FIXME: Walk attributes here.

        if let Some(ty) = &node.data_type {
            visitor.visit_data_type(ty);
        }

        if let Some(default) = &node.default {
            visitor.visit_expression(default);
        }
    }

    walk_function_body: FunctionBody => {
        visitor.visit(&node.statements);
    }

    walk_class: ClassStatement => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.
        visitor.visit_simple_identifier(&node.name);

        if let Some(extends) = &node.extends {
            visitor.visit_class_extends(extends);
        }

        if let Some(implements) = &node.implements {
            visitor.visit_class_implements(implements);
        }

        visitor.visit_class_body(&node.body);
    }

    walk_class_extends: ClassExtends => {
        visitor.visit_simple_identifier(&node.parent);
    }

    walk_class_implements: ClassImplements => {
        for interface in node.interfaces.iter() {
            visitor.visit_simple_identifier(interface);
        }
    }

    walk_class_body: ClassBody => {
        for member in node.members.iter() {
            visitor.visit_classish_member(member);
        }
    }

    walk_classish_member: ClassishMember => {
        match node {
            ClassishMember::Constant(constant) => {
                visitor.visit_classish_constant(&constant);
            },
            ClassishMember::TraitUsage(usage) => {
                visitor.visit_trait_usage(&usage);
            },
            ClassishMember::Property(property) => {
                visitor.visit_property(&property);
            },
            ClassishMember::VariableProperty(property) => {
                visitor.visit_variable_property(&property);
            },
            ClassishMember::AbstractMethod(method) => {
                visitor.visit_abstract_method(&method);
            },
            ClassishMember::AbstractConstructor(method) => {
                visitor.visit_abstract_constructor(&method);
            },
            ClassishMember::ConcreteMethod(method) => {
                visitor.visit_concrete_method(&method);
            },
            ClassishMember::ConcreteConstructor(method) => {
                visitor.visit_concrete_constructor(&method);
            },
        }
    }

    walk_classish_constant: ClassishConstant => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        for entries in node.entries.iter() {
            visitor.visit_constant_entry(entries);
        }
    }

    walk_trait_usage: TraitUsage => {
        for r#trait in node.traits.iter() {
            visitor.visit_simple_identifier(r#trait);
        }

        for adaptation in node.adaptations.iter() {
            visitor.visit_trait_usage_adaptation(adaptation);
        }
    }

    walk_trait_usage_adaptation: TraitUsageAdaptation => {
        match node {
            TraitUsageAdaptation::Alias { r#trait, method, alias, visibility } => {
                if let Some(r#trait) = r#trait {
                    visitor.visit_simple_identifier(&r#trait);
                }

                visitor.visit_simple_identifier(&method);
                visitor.visit_simple_identifier(&alias);

                if let Some(_visibility) = visibility {
                    // FIXME: Visit visibility here.
                    // visitor.visit_visibility_modifier(visibility);
                }
            },
            TraitUsageAdaptation::Visibility { r#trait, method, visibility: _visibility } => {
                if let Some(r#trait) = r#trait {
                    visitor.visit_simple_identifier(&r#trait);
                }

                visitor.visit_simple_identifier(&method);
                // FIXME: Visit visibility here.
            },
            TraitUsageAdaptation::Precedence { r#trait, method, insteadof } => {
                if let Some(r#trait) = r#trait {
                    visitor.visit_simple_identifier(&r#trait);
                }

                visitor.visit_simple_identifier(&method);

                for insteadof in insteadof.iter() {
                    visitor.visit_simple_identifier(insteadof);
                }
            }
        }
    }

    walk_property: Property => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        if let Some(ty) = &node.r#type {
            visitor.visit_data_type(ty);
        }

        for entry in node.entries.iter() {
            visitor.visit_property_entry(entry);
        }
    }

    walk_property_entry: PropertyEntry => {
        match node {
            PropertyEntry::Uninitialized { variable } => {
                visitor.visit_simple_variable(&variable);
            },
            PropertyEntry::Initialized { variable, value, .. } => {
                visitor.visit_simple_variable(&variable);
                visitor.visit_expression(&value);
            },
        }
    }

    walk_variable_property: VariableProperty => {
        // FIXME: Walk attributes here.
        // FIXME: Walk type here.

        for entry in node.entries.iter() {
            visitor.visit_property_entry(entry);
        }
    }

    walk_abstract_method: AbstractMethod => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_simple_identifier(&node.name);
        visitor.visit_function_parameter_list(&node.parameters);

        if let Some(ty) = &node.return_type {
            visitor.visit_return_type(ty);
        }
    }

    walk_abstract_constructor: AbstractConstructor => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_constructor_parameter_list(&node.parameters);
    }

    walk_constructor_parameter_list: ConstructorParameterList => {
        for parameter in node.parameters.iter() {
            visitor.visit_constructor_parameter(parameter);
        }
    }

    walk_constructor_parameter: ConstructorParameter => {
        // FIXME: Walk attributes here.
        // FIXME: Visit modifiers here.
        // FIXME: Visit type here.
        visitor.visit_simple_variable(&node.name);

        if let Some(default) = &node.default {
            visitor.visit_expression(default);
        }
    }

    walk_concrete_method: ConcreteMethod => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_simple_identifier(&node.name);
        visitor.visit_function_parameter_list(&node.parameters);

        if let Some(ty) = &node.return_type {
            visitor.visit_return_type(ty);
        }

        visitor.visit_method_body(&node.body);
    }

    walk_method_body: MethodBody => {
        visitor.visit(&node.statements);
    }

    walk_concrete_constructor: ConcreteConstructor => {
        // FIXME: Walk attributes here.
        // FIXME: Walk modifiers here.

        visitor.visit_constructor_parameter_list(&node.parameters);
        visitor.visit_method_body(&node.body);
    }

    walk_interface: InterfaceStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&node.name);

        if let Some(extends) = &node.extends {
            visitor.visit_interface_extends(extends);
        }

        visitor.visit_interface_body(&node.body);
    }

    walk_interface_extends: InterfaceExtends => {
        for parent in node.parents.iter() {
            visitor.visit_simple_identifier(parent);
        }
    }

    walk_interface_body: InterfaceBody => {
        for member in node.members.iter() {
            visitor.visit_classish_member(member);
        }
    }

    walk_trait: TraitStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&node.name);
        visitor.visit_trait_body(&node.body);
    }

    walk_trait_body: TraitBody => {
        for member in node.members.iter() {
            visitor.visit_classish_member(member);
        }
    }

    walk_echo: EchoStatement => {
        for value in node.values.iter() {
            visitor.visit_expression(value);
        }
    }

    walk_expression_stmt: ExpressionStatement => {
        visitor.visit_expression(&node.expression);
    }

    walk_return: ReturnStatement => {
        if let Some(expression) = &node.value {
            visitor.visit_expression(expression);
        }
    }

    walk_namespace: NamespaceStatement => {
        match node {
            NamespaceStatement::Unbraced(node) => {
                visitor.visit_unbraced_namespace(&node);
            },
            NamespaceStatement::Braced(node) => {
                visitor.visit_braced_namespace(&node);
            },
        }
    }

    walk_unbraced_namespace: UnbracedNamespace => {
        visitor.visit_simple_identifier(&node.name);
        visitor.visit(&node.statements);
    }

    walk_braced_namespace: BracedNamespace => {
        if let Some(name) = &node.name {
            visitor.visit_simple_identifier(name);
        }

        visitor.visit(&node.body.statements);
    }

    walk_use: UseStatement => {
        for r#use in node.uses.iter() {
            visitor.visit_use_use(r#use);
        }
    }

    walk_use_use: Use => {
        visitor.visit_simple_identifier(&node.name);

        if let Some(alias) = &node.alias {
            visitor.visit_simple_identifier(alias);
        }
    }

    walk_group_use: GroupUseStatement => {
        visitor.visit_simple_identifier(&node.prefix);

        for r#use in node.uses.iter() {
            visitor.visit_use_use(r#use);
        }
    }

    walk_try: TryStatement => {
        visitor.visit(&node.body);

        for catch in node.catches.iter() {
            visitor.visit_catch_block(catch);
        }

        if let Some(finally) = &node.finally {
            visitor.visit_finally_block(finally);
        }
    }

    walk_catch_block: CatchBlock => {
        if let Some(variable) = &node.var {
            visitor.visit_simple_variable(variable);
        }

        visitor.visit(&node.body);
    }

    walk_finally_block: FinallyBlock => {
        visitor.visit(&node.body);
    }

    walk_unit_enum: UnitEnumStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&node.name);

        for implements in node.implements.iter() {
            visitor.visit_simple_identifier(implements);
        }

        for member in node.body.members.iter() {
            visitor.visit_unit_enum_member(member);
        }
    }

    walk_unit_enum_member: UnitEnumMember => {
        match node {
            UnitEnumMember::Case(node) => {
                visitor.visit_unit_enum_case(&node);
            }
            UnitEnumMember::Classish(node) => {
                visitor.visit_classish_member(&node);
            }
        }
    }

    walk_unit_enum_case: UnitEnumCase => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&node.name);
    }

    walk_backed_enum: BackedEnumStatement => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&node.name);

        for implements in node.implements.iter() {
            visitor.visit_simple_identifier(implements);
        }

        for member in node.body.members.iter() {
            visitor.visit_backed_enum_member(member);
        }
    }

    walk_backed_enum_member: BackedEnumMember => {
        match node {
            BackedEnumMember::Case(node) => {
                visitor.visit_backed_enum_case(&node);
            }
            BackedEnumMember::Classish(node) => {
                visitor.visit_classish_member(&node);
            }
        }
    }

    walk_backed_enum_case: BackedEnumCase => {
        // FIXME: Walk attributes here.

        visitor.visit_simple_identifier(&node.name);
        visitor.visit_expression(&node.value);
    }

    walk_declare: DeclareStatement => {
        for entry in node.entries.entries.iter() {
            visitor.visit_declare_entry(entry);
        }

        visitor.visit_declare_body(&node.body);
    }

    walk_declare_entry: DeclareEntry => {
        visitor.visit_simple_identifier(&node.key);
        visitor.visit_literal(&node.value);
    }

    walk_declare_body: DeclareBody => {
        match node {
            DeclareBody::Noop { .. } => {},
            DeclareBody::Braced { statements, .. } => {
                visitor.visit(&statements);
            },
            DeclareBody::Expression { expression, .. } => {
                visitor.visit_expression(&expression);
            },
            DeclareBody::Block { statements, .. } => {
                visitor.visit(&statements);
            },
        }
    }

    walk_argument_list: ArgumentList => {
        for argument in node.arguments.iter() {
            visitor.visit_argument(argument);
        }
    }

    walk_argument: Argument => {
        match node {
            Argument::Positional(node) => {
                visitor.visit_expression(&node.value);
            },
            Argument::Named(node) => {
                visitor.visit_simple_identifier(&node.name);
                visitor.visit_expression(&node.value);
            },
        }
    }

    walk_eval: EvalExpression => {
        if let Some(argument) = &node.argument.argument {
            visitor.visit_argument(argument);
        }
    }

    walk_empty: EmptyExpression => {
        if let Some(argument) = &node.argument.argument {
            visitor.visit_argument(argument);
        }
    }

    walk_die: DieExpression => {
        if let Some(argument) = &node.argument {
            if let Some(argument) = &argument.argument {
                visitor.visit_argument(argument);
            }
        }
    }

    walk_exit: ExitExpression => {
        if let Some(argument) = &node.argument {
            if let Some(argument) = &argument.argument {
                visitor.visit_argument(argument);
            }
        }
    }

    walk_isset: IssetExpression => {
        visitor.visit_argument_list(&node.arguments);
    }

    walk_unset: UnsetExpression => {
        visitor.visit_argument_list(&node.arguments);
    }

    walk_print: PrintExpression => {
        if let Some(value) = &node.value {
            visitor.visit_expression(value);
        }

        if let Some(argument) = &node.argument {
            if let Some(argument) = &argument.argument {
                visitor.visit_argument(argument);
            }
        }
    }

    walk_arithmetic_operation: ArithmeticOperationExpression => {
        match node {
            ArithmeticOperationExpression::Addition {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::Subtraction {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::Multiplication {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::Division {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::Modulo {
                left, right, ..
            }  => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::Exponentiation {
                left, right, ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::Negative {
                right, ..
            } => {
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::Positive {
                right,
                ..
            } => {
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::PreIncrement {
                right, ..
            } => {
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::PostIncrement {
                left, ..
            } => {
                visitor.visit_expression(&left);
            },
            ArithmeticOperationExpression::PreDecrement {
                right, ..
            } => {
                visitor.visit_expression(&right);
            },
            ArithmeticOperationExpression::PostDecrement {
                left, ..
            } => {
                visitor.visit_expression(&left);
            },
        }
    }

    walk_assignment_operation: AssignmentOperationExpression => {
        match node {
            AssignmentOperationExpression::Assign {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Addition {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Subtraction {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Multiplication {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Division {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Modulo {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Exponentiation {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Concat {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::BitwiseAnd {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::BitwiseOr {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::BitwiseXor {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::LeftShift {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::RightShift {
                left,
                right,
                ..
            } |
            AssignmentOperationExpression::Coalesce {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            }
        }
    }

    walk_bitwise_operation: BitwiseOperationExpression => {
        match node {
            BitwiseOperationExpression::And {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::Or {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::Xor {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::LeftShift {
                left,
                right,
                ..
            } |
            BitwiseOperationExpression::RightShift {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            BitwiseOperationExpression::Not {
                right,
                ..
            } => {
                visitor.visit_expression(&right);
            },
        }
    }

    walk_comparison_operation: ComparisonOperationExpression => {
        match node {
            ComparisonOperationExpression::Equal {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::Identical {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::NotEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::AngledNotEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::NotIdentical {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::LessThan {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::GreaterThan {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::LessThanOrEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::GreaterThanOrEqual {
                left,
                right,
                ..
            } |
            ComparisonOperationExpression::Spaceship {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            }
        }
    }

    walk_logical_operation: LogicalOperationExpression => {
        match node {
            LogicalOperationExpression::And {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::Or {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::LogicalAnd {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::LogicalOr {
                left,
                right,
                ..
            } |
            LogicalOperationExpression::LogicalXor {
                left,
                right,
                ..
            } => {
                visitor.visit_expression(&left);
                visitor.visit_expression(&right);
            },
            LogicalOperationExpression::Not {
                right,
                ..
            } => {
                visitor.visit_expression(&right);
            },
        }
    }

    walk_concat: ConcatExpression => {
        visitor.visit_expression(&node.left);
        visitor.visit_expression(&node.right);
    }

    walk_instanceof: InstanceofExpression => {
        visitor.visit_expression(&node.left);
        visitor.visit_expression(&node.right);
    }

    walk_reference: ReferenceExpression => {
        visitor.visit_expression(&node.right);
    }

    walk_parenthesized: ParenthesizedExpression => {
        visitor.visit_expression(&node.expr);
    }

    walk_error_suppress: ErrorSuppressExpression => {
        visitor.visit_expression(&node.expr);
    }

    walk_identifier: Identifier => {
        match node {
            Identifier::SimpleIdentifier(node) => {
                visitor.visit_simple_identifier(&node);
            },
            Identifier::DynamicIdentifier(node) => {
                visitor.visit_dynamic_identifier(&node);
            },
        }
    }

    walk_dynamic_identifier: DynamicIdentifier => {
        visitor.visit_expression(&node.expr);
    }

    walk_variable: Variable => {
        match node {
            Variable::SimpleVariable(node) => {
                visitor.visit_simple_variable(&node);
            },
            Variable::VariableVariable(node) => {
                visitor.visit_variable_variable(&node);
            },
            Variable::BracedVariableVariable(node) => {
                visitor.visit_braced_variable_variable(&node);
            }
        }
    }

    walk_variable_variable: VariableVariable => {
        visitor.visit_variable(&node.variable);
    }

    walk_braced_variable_variable: BracedVariableVariable => {
        visitor.visit_expression(&node.variable);
    }

    walk_include: IncludeExpression => {
        visitor.visit_expression(&node.path);
    }

    walk_include_once: IncludeOnceExpression => {
        visitor.visit_expression(&node.path);
    }

    walk_require: RequireExpression => {
        visitor.visit_expression(&node.path);
    }

    walk_require_once: RequireOnceExpression => {
        visitor.visit_expression(&node.path);
    }

    walk_function_call: FunctionCallExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_argument_list(&node.arguments);
    }

    walk_function_closure_creation: FunctionClosureCreationExpression => {
        visitor.visit_expression(&node.target);
    }

    walk_method_call: MethodCallExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_expression(&node.method);
        visitor.visit_argument_list(&node.arguments);
    }

    walk_method_closure_creation: MethodClosureCreationExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_expression(&node.method);
    }

    walk_nullsafe_method_call: NullsafeMethodCallExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_expression(&node.method);
        visitor.visit_argument_list(&node.arguments);
    }

    walk_static_method_call: StaticMethodCallExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_identifier(&node.method);
        visitor.visit_argument_list(&node.arguments);
    }

    walk_static_variable_method_call: StaticVariableMethodCallExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_variable(&node.method);
        visitor.visit_argument_list(&node.arguments);
    }

    walk_static_method_closure_creation: StaticMethodClosureCreationExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_identifier(&node.method);
    }

    walk_static_variable_method_closure_creation: StaticVariableMethodClosureCreationExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_variable(&node.method);
    }

    walk_property_fetch: PropertyFetchExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_expression(&node.property);
    }

    walk_nullsafe_property_fetch: NullsafePropertyFetchExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_expression(&node.property);
    }

    walk_static_property_fetch: StaticPropertyFetchExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_variable(&node.property);
    }

    walk_constant_fetch: ConstantFetchExpression => {
        visitor.visit_expression(&node.target);
        visitor.visit_identifier(&node.constant);
    }

    walk_array_item: ArrayItem => {
        match node {
            ArrayItem::Skipped => {},
            ArrayItem::Value {
                value
            } => {
                visitor.visit_expression(&value);
            },
            ArrayItem::ReferencedValue {
                value, ..
            } => {
                visitor.visit_expression(&value);
            },
            ArrayItem::SpreadValue {
                value, ..
            } => {
                visitor.visit_expression(&value);
            },
            ArrayItem::KeyValue {
                key,
                value,
                ..
            } => {
                visitor.visit_expression(&key);
                visitor.visit_expression(&value);
            },
            ArrayItem::ReferencedKeyValue {
                key,
                value, ..
            } => {
                visitor.visit_expression(&key);
                visitor.visit_expression(&value);
            },
        }
    }

    walk_short_array: ShortArrayExpression => {
        for item in node.items.inner.iter() {
            visitor.visit_array_item(item);
        }
    }

    walk_array: ArrayExpression => {
        for item in node.items.inner.iter() {
            visitor.visit_array_item(item);
        }
    }

    walk_list: ListExpression => {
        for entry in node.items.iter() {
            visitor.visit_list_entry(entry);
        }
    }

    walk_list_entry: ListEntry => {
        match node {
            ListEntry::Skipped => {},
            ListEntry::Value {
                value
            } => {
                visitor.visit_expression(&value);
            },
            ListEntry::KeyValue {
                key,
                value,
                ..
            } => {
                visitor.visit_expression(&key);
                visitor.visit_expression(&value);
            },
        }
    }

    walk_closure: ClosureExpression => {
        // FIXME: Walk attributes here.

        visitor.visit_function_parameter_list(&node.parameters);

        if let Some(return_type) = &node.return_type {
            visitor.visit_return_type(return_type);
        }

        visitor.visit_function_body(&node.body);
    }

    walk_arrow_function: ArrowFunctionExpression => {
        // FIXME: Walk attributes here.

        visitor.visit_function_parameter_list(&node.parameters);
        visitor.visit_expression(&node.body);
    }

    walk_new: NewExpression => {
        visitor.visit_expression(&node.target);

        if let Some(arguments) = &node.arguments {
            visitor.visit_argument_list(arguments);
        }
    }

    walk_interpolated_string: InterpolatedStringExpression => {
        for part in node.parts.iter() {
            visitor.visit_string_part(part);
        }
    }

    walk_string_part: StringPart => {
        match node {
            StringPart::Literal(node) => {
                visitor.visit_literal_string_part(&node);
            },
            StringPart::Expression(node) => {
                visitor.visit_expression_string_part(&node);
            }
        }
    }

    walk_expression_string_part: ExpressionStringPart => {
        visitor.visit_expression(&node.expression);
    }

    walk_heredoc: HeredocExpression => {
        for part in node.parts.iter() {
            visitor.visit_string_part(part);
        }
    }

    walk_shell_exec: ShellExecExpression => {
        for part in node.parts.iter() {
            visitor.visit_string_part(part);
        }
    }

    walk_anonymous_class: AnonymousClassExpression => {
        // FIXME: Walk attributes here.

        if let Some(extends) = &node.extends {
            visitor.visit_class_extends(extends);
        }

        if let Some(implements) = &node.implements {
            visitor.visit_class_implements(implements);
        }

        visitor.visit_anonymous_class_body(&node.body);
    }

    walk_anonymous_class_body: AnonymousClassBody => {
        for member in node.members.iter() {
            visitor.visit_classish_member(member);
        }
    }

    walk_array_index: ArrayIndexExpression => {
        visitor.visit_expression(&node.array);

        if let Some(index) = &node.index {
            visitor.visit_expression(index);
        }
    }

    walk_short_ternary: ShortTernaryExpression => {
        visitor.visit_expression(&node.condition);
        visitor.visit_expression(&node.r#else);
    }

    walk_ternary: TernaryExpression => {
        visitor.visit_expression(&node.condition);
        visitor.visit_expression(&node.then);
        visitor.visit_expression(&node.r#else);
    }

    walk_coalesce: CoalesceExpression => {
        visitor.visit_expression(&node.lhs);
        visitor.visit_expression(&node.rhs);
    }

    walk_clone: CloneExpression => {
        visitor.visit_expression(&node.target);
    }

    walk_match: MatchExpression => {
        visitor.visit_expression(&node.condition);

        for arm in node.arms.iter() {
            visitor.visit_match_arm(arm);
        }

        if let Some(default) = &node.default {
            visitor.visit_default_match_arm(default);
        }
    }

    walk_match_arm: MatchArm => {
        for condition in node.conditions.iter() {
            visitor.visit_expression(condition);
        }

        visitor.visit_expression(&node.body);
    }

    walk_default_match_arm: DefaultMatchArm => {
        visitor.visit_expression(&node.body);
    }

    walk_throw: ThrowExpression => {
        visitor.visit_expression(&node.value);
    }

    walk_yield: YieldExpression => {
        if let Some(key) = &node.key {
            visitor.visit_expression(key);
        }

        if let Some(value) = &node.value {
            visitor.visit_expression(value);
        }
    }

    walk_yield_from: YieldFromExpression => {
        visitor.visit_expression(&node.value);
    }

    walk_cast: CastExpression => {
        visitor.visit_expression(&node.value);
    }

    walk_return_type: ReturnType => {
        visitor.visit_data_type(&node.data_type);
    }

    walk_data_type: DataType => {
        visitor.visit_type(&node.kind);
    }

    walk_type: Type => {
        match node {
            Type::Nullable(ty) => {
                visitor.visit_type(&ty);
            },
            Type::Union(tys) |
            Type::Intersection(tys) => {
                for ty in tys.iter() {
                    visitor.visit_type(ty);
                }
            },
            _ => {},
        }
    }
}
