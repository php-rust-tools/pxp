use pxp_ast::*;
use crate::walk::*;
use pxp_span::Span;
use pxp_syntax::comments::Comment;
use pxp_type::Type;

pub trait VisitorMut {
    fn visit(&mut self, node: &mut [Statement]) {
        walk_mut(self, node);
    }

    fn visit_statement(&mut self, node: &mut Statement) {
        walk_statement_mut(self, node);
    }

    fn visit_expression(&mut self, node: &mut Expression) {
        walk_expression_mut(self, node);
    }

    fn visit_inline_html_statement(&mut self, node: &mut InlineHtmlStatement) {
}


fn visit_full_opening_tag_statement(&mut self, node: &mut FullOpeningTagStatement) {
}


fn visit_short_opening_tag_statement(&mut self, node: &mut ShortOpeningTagStatement) {
}


fn visit_echo_opening_tag_statement(&mut self, node: &mut EchoOpeningTagStatement) {
}


fn visit_closing_tag_statement(&mut self, node: &mut ClosingTagStatement) {
}


fn visit_expression_statement(&mut self, node: &mut ExpressionStatement) {
   walk_expression_statement(self, node);
}


fn visit_global_statement(&mut self, node: &mut GlobalStatement) {
   walk_global_statement(self, node);
}


fn visit_block_statement(&mut self, node: &mut BlockStatement) {
   walk_block_statement(self, node);
}


fn visit_cast_kind(&mut self, node: &mut CastKind) {
   walk_cast_kind(self, node);
}


fn visit_case(&mut self, node: &mut Case) {
   walk_case(self, node);
}


fn visit_use(&mut self, node: &mut Use) {
   walk_use(self, node);
}


fn visit_use_kind(&mut self, node: &mut UseKind) {
   walk_use_kind(self, node);
}


fn visit_eval_expression(&mut self, node: &mut EvalExpression) {
   walk_eval_expression(self, node);
}


fn visit_empty_expression(&mut self, node: &mut EmptyExpression) {
   walk_empty_expression(self, node);
}


fn visit_die_expression(&mut self, node: &mut DieExpression) {
   walk_die_expression(self, node);
}


fn visit_exit_expression(&mut self, node: &mut ExitExpression) {
   walk_exit_expression(self, node);
}


fn visit_isset_expression(&mut self, node: &mut IssetExpression) {
   walk_isset_expression(self, node);
}


fn visit_unset_expression(&mut self, node: &mut UnsetExpression) {
   walk_unset_expression(self, node);
}


fn visit_print_expression(&mut self, node: &mut PrintExpression) {
   walk_print_expression(self, node);
}


fn visit_concat_expression(&mut self, node: &mut ConcatExpression) {
   walk_concat_expression(self, node);
}


fn visit_instanceof_expression(&mut self, node: &mut InstanceofExpression) {
   walk_instanceof_expression(self, node);
}


fn visit_reference_expression(&mut self, node: &mut ReferenceExpression) {
   walk_reference_expression(self, node);
}


fn visit_parenthesized_expression(&mut self, node: &mut ParenthesizedExpression) {
   walk_parenthesized_expression(self, node);
}


fn visit_error_suppress_expression(&mut self, node: &mut ErrorSuppressExpression) {
   walk_error_suppress_expression(self, node);
}


fn visit_include_expression(&mut self, node: &mut IncludeExpression) {
   walk_include_expression(self, node);
}


fn visit_include_once_expression(&mut self, node: &mut IncludeOnceExpression) {
   walk_include_once_expression(self, node);
}


fn visit_require_expression(&mut self, node: &mut RequireExpression) {
   walk_require_expression(self, node);
}


fn visit_require_once_expression(&mut self, node: &mut RequireOnceExpression) {
   walk_require_once_expression(self, node);
}


fn visit_function_call_expression(&mut self, node: &mut FunctionCallExpression) {
   walk_function_call_expression(self, node);
}


fn visit_function_closure_creation_expression(&mut self, node: &mut FunctionClosureCreationExpression) {
   walk_function_closure_creation_expression(self, node);
}


fn visit_method_call_expression(&mut self, node: &mut MethodCallExpression) {
   walk_method_call_expression(self, node);
}


fn visit_method_closure_creation_expression(&mut self, node: &mut MethodClosureCreationExpression) {
   walk_method_closure_creation_expression(self, node);
}


fn visit_nullsafe_method_call_expression(&mut self, node: &mut NullsafeMethodCallExpression) {
   walk_nullsafe_method_call_expression(self, node);
}


fn visit_static_method_call_expression(&mut self, node: &mut StaticMethodCallExpression) {
   walk_static_method_call_expression(self, node);
}


fn visit_static_variable_method_call_expression(&mut self, node: &mut StaticVariableMethodCallExpression) {
   walk_static_variable_method_call_expression(self, node);
}


fn visit_static_method_closure_creation_expression(&mut self, node: &mut StaticMethodClosureCreationExpression) {
   walk_static_method_closure_creation_expression(self, node);
}


fn visit_static_variable_method_closure_creation_expression(&mut self, node: &mut StaticVariableMethodClosureCreationExpression) {
   walk_static_variable_method_closure_creation_expression(self, node);
}


fn visit_property_fetch_expression(&mut self, node: &mut PropertyFetchExpression) {
   walk_property_fetch_expression(self, node);
}


fn visit_nullsafe_property_fetch_expression(&mut self, node: &mut NullsafePropertyFetchExpression) {
   walk_nullsafe_property_fetch_expression(self, node);
}


fn visit_static_property_fetch_expression(&mut self, node: &mut StaticPropertyFetchExpression) {
   walk_static_property_fetch_expression(self, node);
}


fn visit_constant_fetch_expression(&mut self, node: &mut ConstantFetchExpression) {
   walk_constant_fetch_expression(self, node);
}


fn visit_short_array_expression(&mut self, node: &mut ShortArrayExpression) {
   walk_short_array_expression(self, node);
}


fn visit_array_expression(&mut self, node: &mut ArrayExpression) {
   walk_array_expression(self, node);
}


fn visit_list_expression(&mut self, node: &mut ListExpression) {
   walk_list_expression(self, node);
}


fn visit_new_expression(&mut self, node: &mut NewExpression) {
   walk_new_expression(self, node);
}


fn visit_interpolated_string_expression(&mut self, node: &mut InterpolatedStringExpression) {
   walk_interpolated_string_expression(self, node);
}


fn visit_heredoc_expression(&mut self, node: &mut HeredocExpression) {
   walk_heredoc_expression(self, node);
}


fn visit_nowdoc_expression(&mut self, node: &mut NowdocExpression) {
}


fn visit_shell_exec_expression(&mut self, node: &mut ShellExecExpression) {
   walk_shell_exec_expression(self, node);
}


fn visit_bool_expression(&mut self, node: &mut BoolExpression) {
   walk_bool_expression(self, node);
}


fn visit_array_index_expression(&mut self, node: &mut ArrayIndexExpression) {
   walk_array_index_expression(self, node);
}


fn visit_short_ternary_expression(&mut self, node: &mut ShortTernaryExpression) {
   walk_short_ternary_expression(self, node);
}


fn visit_ternary_expression(&mut self, node: &mut TernaryExpression) {
   walk_ternary_expression(self, node);
}


fn visit_coalesce_expression(&mut self, node: &mut CoalesceExpression) {
   walk_coalesce_expression(self, node);
}


fn visit_clone_expression(&mut self, node: &mut CloneExpression) {
   walk_clone_expression(self, node);
}


fn visit_match_expression(&mut self, node: &mut MatchExpression) {
   walk_match_expression(self, node);
}


fn visit_throw_expression(&mut self, node: &mut ThrowExpression) {
   walk_throw_expression(self, node);
}


fn visit_yield_expression(&mut self, node: &mut YieldExpression) {
   walk_yield_expression(self, node);
}


fn visit_yield_from_expression(&mut self, node: &mut YieldFromExpression) {
   walk_yield_from_expression(self, node);
}


fn visit_cast_expression(&mut self, node: &mut CastExpression) {
   walk_cast_expression(self, node);
}


fn visit_default_match_arm(&mut self, node: &mut DefaultMatchArm) {
   walk_default_match_arm(self, node);
}


fn visit_match_arm(&mut self, node: &mut MatchArm) {
   walk_match_arm(self, node);
}


fn visit_magic_constant_expression(&mut self, node: &mut MagicConstantExpression) {
}


fn visit_string_part(&mut self, node: &mut StringPart) {
   walk_string_part(self, node);
}


fn visit_literal_string_part(&mut self, node: &mut LiteralStringPart) {
   walk_literal_string_part(self, node);
}


fn visit_expression_string_part(&mut self, node: &mut ExpressionStringPart) {
   walk_expression_string_part(self, node);
}


fn visit_array_item(&mut self, node: &mut ArrayItem) {
   walk_array_item(self, node);
}


fn visit_list_entry(&mut self, node: &mut ListEntry) {
   walk_list_entry(self, node);
}


fn visit_positional_argument(&mut self, node: &mut PositionalArgument) {
   walk_positional_argument(self, node);
}


fn visit_named_argument(&mut self, node: &mut NamedArgument) {
   walk_named_argument(self, node);
}


fn visit_argument(&mut self, node: &mut Argument) {
   walk_argument(self, node);
}


fn visit_argument_list(&mut self, node: &mut ArgumentList) {
   walk_argument_list(self, node);
}


fn visit_single_argument(&mut self, node: &mut SingleArgument) {
   walk_single_argument(self, node);
}


fn visit_argument_placeholder(&mut self, node: &mut ArgumentPlaceholder) {
   walk_argument_placeholder(self, node);
}


fn visit_attribute(&mut self, node: &mut Attribute) {
   walk_attribute(self, node);
}


fn visit_attribute_group(&mut self, node: &mut AttributeGroup) {
   walk_attribute_group(self, node);
}


fn visit_class_body(&mut self, node: &mut ClassBody) {
   walk_class_body(self, node);
}


fn visit_class_statement(&mut self, node: &mut ClassStatement) {
   walk_class_statement(self, node);
}


fn visit_anonymous_class_body(&mut self, node: &mut AnonymousClassBody) {
   walk_anonymous_class_body(self, node);
}


fn visit_anonymous_class_expression(&mut self, node: &mut AnonymousClassExpression) {
   walk_anonymous_class_expression(self, node);
}


fn visit_class_extends(&mut self, node: &mut ClassExtends) {
   walk_class_extends(self, node);
}


fn visit_class_implements(&mut self, node: &mut ClassImplements) {
   walk_class_implements(self, node);
}


fn visit_classish_member(&mut self, node: &mut ClassishMember) {
   walk_classish_member(self, node);
}


fn visit_constant_entry(&mut self, node: &mut ConstantEntry) {
   walk_constant_entry(self, node);
}


fn visit_constant_statement(&mut self, node: &mut ConstantStatement) {
   walk_constant_statement(self, node);
}


fn visit_classish_constant(&mut self, node: &mut ClassishConstant) {
   walk_classish_constant(self, node);
}


fn visit_if_statement(&mut self, node: &mut IfStatement) {
   walk_if_statement(self, node);
}


fn visit_if_statement_body(&mut self, node: &mut IfStatementBody) {
   walk_if_statement_body(self, node);
}


fn visit_if_statement_else_if(&mut self, node: &mut IfStatementElseIf) {
   walk_if_statement_else_if(self, node);
}


fn visit_if_statement_else(&mut self, node: &mut IfStatementElse) {
   walk_if_statement_else(self, node);
}


fn visit_if_statement_else_if_block(&mut self, node: &mut IfStatementElseIfBlock) {
   walk_if_statement_else_if_block(self, node);
}


fn visit_if_statement_else_block(&mut self, node: &mut IfStatementElseBlock) {
   walk_if_statement_else_block(self, node);
}


fn visit_data_type(&mut self, node: &mut DataType) {
   walk_data_type(self, node);
}


fn visit_declare_entry(&mut self, node: &mut DeclareEntry) {
   walk_declare_entry(self, node);
}


fn visit_declare_entry_group(&mut self, node: &mut DeclareEntryGroup) {
   walk_declare_entry_group(self, node);
}


fn visit_declare_body(&mut self, node: &mut DeclareBody) {
   walk_declare_body(self, node);
}


fn visit_declare_statement(&mut self, node: &mut DeclareStatement) {
   walk_declare_statement(self, node);
}


fn visit_unit_enum_case(&mut self, node: &mut UnitEnumCase) {
   walk_unit_enum_case(self, node);
}


fn visit_unit_enum_member(&mut self, node: &mut UnitEnumMember) {
   walk_unit_enum_member(self, node);
}


fn visit_unit_enum_body(&mut self, node: &mut UnitEnumBody) {
   walk_unit_enum_body(self, node);
}


fn visit_unit_enum_statement(&mut self, node: &mut UnitEnumStatement) {
   walk_unit_enum_statement(self, node);
}


fn visit_backed_enum_case(&mut self, node: &mut BackedEnumCase) {
   walk_backed_enum_case(self, node);
}


fn visit_backed_enum_member(&mut self, node: &mut BackedEnumMember) {
   walk_backed_enum_member(self, node);
}


fn visit_backed_enum_body(&mut self, node: &mut BackedEnumBody) {
   walk_backed_enum_body(self, node);
}


fn visit_backed_enum_statement(&mut self, node: &mut BackedEnumStatement) {
   walk_backed_enum_statement(self, node);
}


fn visit_return_type(&mut self, node: &mut ReturnType) {
   walk_return_type(self, node);
}


fn visit_function_parameter(&mut self, node: &mut FunctionParameter) {
   walk_function_parameter(self, node);
}


fn visit_function_parameter_list(&mut self, node: &mut FunctionParameterList) {
   walk_function_parameter_list(self, node);
}


fn visit_function_body(&mut self, node: &mut FunctionBody) {
   walk_function_body(self, node);
}


fn visit_function_statement(&mut self, node: &mut FunctionStatement) {
   walk_function_statement(self, node);
}


fn visit_closure_use_variable(&mut self, node: &mut ClosureUseVariable) {
   walk_closure_use_variable(self, node);
}


fn visit_closure_use(&mut self, node: &mut ClosureUse) {
   walk_closure_use(self, node);
}


fn visit_closure_expression(&mut self, node: &mut ClosureExpression) {
   walk_closure_expression(self, node);
}


fn visit_arrow_function_expression(&mut self, node: &mut ArrowFunctionExpression) {
   walk_arrow_function_expression(self, node);
}


fn visit_constructor_parameter(&mut self, node: &mut ConstructorParameter) {
   walk_constructor_parameter(self, node);
}


fn visit_constructor_parameter_list(&mut self, node: &mut ConstructorParameterList) {
   walk_constructor_parameter_list(self, node);
}


fn visit_abstract_constructor(&mut self, node: &mut AbstractConstructor) {
   walk_abstract_constructor(self, node);
}


fn visit_concrete_constructor(&mut self, node: &mut ConcreteConstructor) {
   walk_concrete_constructor(self, node);
}


fn visit_abstract_method(&mut self, node: &mut AbstractMethod) {
   walk_abstract_method(self, node);
}


fn visit_concrete_method(&mut self, node: &mut ConcreteMethod) {
   walk_concrete_method(self, node);
}


fn visit_method_body(&mut self, node: &mut MethodBody) {
   walk_method_body(self, node);
}


fn visit_label_statement(&mut self, node: &mut LabelStatement) {
   walk_label_statement(self, node);
}


fn visit_goto_statement(&mut self, node: &mut GotoStatement) {
   walk_goto_statement(self, node);
}


fn visit_identifier(&mut self, node: &mut Identifier) {
   walk_identifier(self, node);
}


fn visit_simple_identifier(&mut self, node: &mut SimpleIdentifier) {
   walk_simple_identifier(self, node);
}


fn visit_dynamic_identifier(&mut self, node: &mut DynamicIdentifier) {
   walk_dynamic_identifier(self, node);
}


fn visit_interface_extends(&mut self, node: &mut InterfaceExtends) {
   walk_interface_extends(self, node);
}


fn visit_interface_body(&mut self, node: &mut InterfaceBody) {
   walk_interface_body(self, node);
}


fn visit_interface_statement(&mut self, node: &mut InterfaceStatement) {
   walk_interface_statement(self, node);
}


fn visit_literal(&mut self, node: &mut Literal) {
   walk_literal(self, node);
}


fn visit_literal_kind(&mut self, node: &mut LiteralKind) {
   walk_literal_kind(self, node);
}


fn visit_foreach_statement(&mut self, node: &mut ForeachStatement) {
   walk_foreach_statement(self, node);
}


fn visit_foreach_statement_iterator(&mut self, node: &mut ForeachStatementIterator) {
   walk_foreach_statement_iterator(self, node);
}


fn visit_foreach_statement_body(&mut self, node: &mut ForeachStatementBody) {
   walk_foreach_statement_body(self, node);
}


fn visit_for_statement(&mut self, node: &mut ForStatement) {
   walk_for_statement(self, node);
}


fn visit_for_statement_iterator(&mut self, node: &mut ForStatementIterator) {
   walk_for_statement_iterator(self, node);
}


fn visit_for_statement_body(&mut self, node: &mut ForStatementBody) {
   walk_for_statement_body(self, node);
}


fn visit_do_while_statement(&mut self, node: &mut DoWhileStatement) {
   walk_do_while_statement(self, node);
}


fn visit_while_statement(&mut self, node: &mut WhileStatement) {
   walk_while_statement(self, node);
}


fn visit_while_statement_body(&mut self, node: &mut WhileStatementBody) {
   walk_while_statement_body(self, node);
}


fn visit_level(&mut self, node: &mut Level) {
   walk_level(self, node);
}


fn visit_break_statement(&mut self, node: &mut BreakStatement) {
   walk_break_statement(self, node);
}


fn visit_continue_statement(&mut self, node: &mut ContinueStatement) {
   walk_continue_statement(self, node);
}


fn visit_visibility_modifier(&mut self, node: &mut VisibilityModifier) {
}


fn visit_promoted_property_modifier(&mut self, node: &mut PromotedPropertyModifier) {
}


fn visit_promoted_property_modifier_group(&mut self, node: &mut PromotedPropertyModifierGroup) {
   walk_promoted_property_modifier_group(self, node);
}


fn visit_property_modifier(&mut self, node: &mut PropertyModifier) {
}


fn visit_property_modifier_group(&mut self, node: &mut PropertyModifierGroup) {
   walk_property_modifier_group(self, node);
}


fn visit_method_modifier(&mut self, node: &mut MethodModifier) {
}


fn visit_method_modifier_group(&mut self, node: &mut MethodModifierGroup) {
   walk_method_modifier_group(self, node);
}


fn visit_class_modifier(&mut self, node: &mut ClassModifier) {
}


fn visit_class_modifier_group(&mut self, node: &mut ClassModifierGroup) {
   walk_class_modifier_group(self, node);
}


fn visit_constant_modifier(&mut self, node: &mut ConstantModifier) {
}


fn visit_constant_modifier_group(&mut self, node: &mut ConstantModifierGroup) {
   walk_constant_modifier_group(self, node);
}


fn visit_unbraced_namespace(&mut self, node: &mut UnbracedNamespace) {
   walk_unbraced_namespace(self, node);
}


fn visit_braced_namespace(&mut self, node: &mut BracedNamespace) {
   walk_braced_namespace(self, node);
}


fn visit_braced_namespace_body(&mut self, node: &mut BracedNamespaceBody) {
   walk_braced_namespace_body(self, node);
}


fn visit_namespace_statement(&mut self, node: &mut NamespaceStatement) {
   walk_namespace_statement(self, node);
}


fn visit_arithmetic_operation_expression(&mut self, node: &mut ArithmeticOperationExpression) {
   walk_arithmetic_operation_expression(self, node);
}


fn visit_assignment_operation_expression(&mut self, node: &mut AssignmentOperationExpression) {
   walk_assignment_operation_expression(self, node);
}


fn visit_bitwise_operation_expression(&mut self, node: &mut BitwiseOperationExpression) {
   walk_bitwise_operation_expression(self, node);
}


fn visit_comparison_operation_expression(&mut self, node: &mut ComparisonOperationExpression) {
   walk_comparison_operation_expression(self, node);
}


fn visit_logical_operation_expression(&mut self, node: &mut LogicalOperationExpression) {
   walk_logical_operation_expression(self, node);
}


fn visit_name(&mut self, node: &mut Name) {
   walk_name(self, node);
}


fn visit_name_kind(&mut self, node: &mut NameKind) {
   walk_name_kind(self, node);
}


fn visit_special_name(&mut self, node: &mut SpecialName) {
   walk_special_name(self, node);
}


fn visit_special_name_kind(&mut self, node: &mut SpecialNameKind) {
   walk_special_name_kind(self, node);
}


fn visit_unresolved_name(&mut self, node: &mut UnresolvedName) {
   walk_unresolved_name(self, node);
}


fn visit_resolved_name(&mut self, node: &mut ResolvedName) {
   walk_resolved_name(self, node);
}


fn visit_property(&mut self, node: &mut Property) {
   walk_property(self, node);
}


fn visit_variable_property(&mut self, node: &mut VariableProperty) {
   walk_variable_property(self, node);
}


fn visit_property_entry(&mut self, node: &mut PropertyEntry) {
   walk_property_entry(self, node);
}


fn visit_trait_body(&mut self, node: &mut TraitBody) {
   walk_trait_body(self, node);
}


fn visit_trait_statement(&mut self, node: &mut TraitStatement) {
   walk_trait_statement(self, node);
}


fn visit_trait_usage(&mut self, node: &mut TraitUsage) {
   walk_trait_usage(self, node);
}


fn visit_trait_usage_adaptation(&mut self, node: &mut TraitUsageAdaptation) {
   walk_trait_usage_adaptation(self, node);
}


fn visit_catch_type(&mut self, node: &mut CatchType) {
   walk_catch_type(self, node);
}


fn visit_try_statement(&mut self, node: &mut TryStatement) {
   walk_try_statement(self, node);
}


fn visit_catch_block(&mut self, node: &mut CatchBlock) {
   walk_catch_block(self, node);
}


fn visit_finally_block(&mut self, node: &mut FinallyBlock) {
   walk_finally_block(self, node);
}


fn visit_variable(&mut self, node: &mut Variable) {
   walk_variable(self, node);
}


fn visit_simple_variable(&mut self, node: &mut SimpleVariable) {
   walk_simple_variable(self, node);
}


fn visit_variable_variable(&mut self, node: &mut VariableVariable) {
   walk_variable_variable(self, node);
}


fn visit_braced_variable_variable(&mut self, node: &mut BracedVariableVariable) {
   walk_braced_variable_variable(self, node);
}


fn visit_ending(&mut self, node: &mut Ending) {
}


fn visit_static_statement(&mut self, node: &mut StaticStatement) {
   walk_static_statement(self, node);
}


fn visit_switch_statement(&mut self, node: &mut SwitchStatement) {
   walk_switch_statement(self, node);
}


fn visit_echo_statement(&mut self, node: &mut EchoStatement) {
   walk_echo_statement(self, node);
}


fn visit_return_statement(&mut self, node: &mut ReturnStatement) {
   walk_return_statement(self, node);
}


fn visit_use_statement(&mut self, node: &mut UseStatement) {
   walk_use_statement(self, node);
}


fn visit_group_use_statement(&mut self, node: &mut GroupUseStatement) {
   walk_group_use_statement(self, node);
}


fn visit_halt_compiler_statement(&mut self, node: &mut HaltCompilerStatement) {
}


fn visit_static_var(&mut self, node: &mut StaticVar) {
   walk_static_var(self, node);
}

}