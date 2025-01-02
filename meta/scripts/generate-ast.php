<?php

use Illuminate\Support\Arr;
use Illuminate\Support\Str;
use Symfony\Component\Yaml\Yaml;

require_once __DIR__ . '/../vendor/autoload.php';

$ast = Yaml::parseFile(__DIR__ . '/../../crates/ast/meta/ast.yaml');
$output = <<<'RUST'
#![allow(unreachable_code, unreachable_patterns, unused_variables, clippy::single_match, clippy::collapsible_match)]
// This file is generated by meta/scripts/generate-ast.php.
// Do not make modifications to this file directly.

use crate::{HasId, utils::CommaSeparated, Node, name::NameQualification};
use pxp_type::Type;
use pxp_token::OwnedToken;
use pxp_span::{Span, Spanned};
use pxp_bytestring::ByteString;
use std::ptr::NonNull;


RUST;

$reserved = ['as', 'derive', 'node', 'children', 'feature', 'rename'];

function is_spanned(string $node, array $structure): bool {
    if (isset($structure['span'])) {
        return true;
    }

    if ($node === 'StatementKind' || $node === 'ExpressionKind') {
        return false;
    }

    foreach ($structure as $field => $value) {
        if (is_array($value) && isset($value['span'])) {
            return true;
        }

        if (is_string($value) && $value === 'Span') {
            return true;
        }
    }

    return false;
}

function feature_flag($structure): string {
    if (! is_array($structure)) {
        return '';
    }

    if (! isset($structure['feature'])) {
        return '';
    }

    $feature = $structure['feature'];

    if (str_starts_with($feature, '!')) {
        return "#[cfg(not(feature = \"" . substr($feature, 1) . "\"))]\n";
    }

    return "#[cfg(feature = \"{$feature}\")]\n";
}

foreach ($ast as $node => $structure) {
    if (is_string($structure)) {
        $output .= "pub type {$node} = {$structure};\n\n";
        continue;
    }

    $output .= feature_flag($structure);

    $derive = 'Debug, PartialEq, Eq, Clone';

    if (is_array($structure) && isset($structure['derive'])) {
        $derive .= ', ' . $structure['derive'];
    }

    $output .= "#[derive({$derive})]\n";
    $enum = isset($structure['as']) && $structure['as'] === 'Enum';

    if (isset($structure['rename'])) {
        $node = $structure['rename'];
    }

    if ($enum) {
        $output .= "pub enum {$node} {\n";
    } else {
        $output .= "pub struct {$node} {\n";
    }

    if ($enum) {
        foreach ($structure as $field => $value) {
            if (in_array($field, $reserved, true)) {
                continue;
            }

            $output .= "    {$field}";

            if ($value === '') {
                $output .= ",\n";
            } elseif (is_string($value)) {
                $output .= "({$value}),\n";
            } elseif (is_array($value)) {
                $output .= " {\n";

                if (! isset($value['node'])) {
                    $output .= "id: NodeId,\n";    
                }

                foreach ($value as $subfield => $subtype) {
                    $output .= "        {$subfield}: {$subtype},\n";
                }

                $output .= "    },\n";
            }
        }
    } else {
        if (! isset($structure['node'])) {
            $output .= "    pub id: NodeId,\n";
        }
        
        foreach ($structure as $field => $type) {
            if (in_array($field, $reserved, true)) {
                continue;
            }

            $output .= "    pub {$field}: {$type},\n";
        }
    }

    $output .= "}\n\n";

    if (!isset($structure['node'])) {
        $output .= feature_flag($structure);
        $output .= "impl HasId for {$node} {\n";
        $output .= "    fn id(&self) -> NodeId {\n";

        if ($enum) {
            $output .= "match self {\n";
                foreach ($structure as $variant => $inner) {
                    if (in_array($variant, $reserved, true)) {
                        continue;
                    }

                    if ($inner === 'Span') {
                        $output .= "{$node}::{$variant}(_) => 0,\n";
                    } elseif (is_array($inner)) {
                        $output .= "{$node}::{$variant} { id, .. } => *id,\n";
                    } else {
                        $output .= "{$node}::{$variant}(inner) => inner.id(),\n";
                    }
                }
            $output .= "}\n";
        } else {
            $output .= "        self.id\n";
        }         

        $output .= "    }\n";
        $output .= "}\n\n";
    }

    if (! is_spanned($node, $structure)) {
        continue;
    }

    $output .= feature_flag($structure);
    $output .= "impl Spanned for {$node} {\n";
    $output .= "    fn span(&self) -> Span {\n";
    
    if (isset($structure['span'])) {
        $output .= "        self.span";
    } elseif ($enum) {
        $output .= "        match self {";
        foreach ($structure as $field => $value) {
            if (in_array($field, $reserved, true)) {
                continue;
            }

            if (is_string($value) && $value === 'Span') {
                $output .= "{$node}::{$field}(span) => *span,\n";
            } elseif (is_array($value)) {
                $output .= "{$node}::{$field} { span, .. } => *span,";
            }
        }
        $output .= "_ => Span::default(),\n";
        $output .= "        }";
    }

    $output .= "    }";
    $output .= "}\n\n";
}

$output .= "#[derive(Debug, PartialEq, Clone, Copy)]\n";
$output .= "pub enum NodeKind<'a> {\n";

foreach ($ast as $node => $structure) {
    if ($node === 'NodeId') {
        continue;
    }

    if (isset($structure['node']) && $structure['node'] === false) {
        continue;
    }

    $output .= feature_flag($structure);

    if (isset($structure['rename'])) {
        $node = $structure['rename'];
    }

    $output .= "    {$node}(&'a {$node}),\n";
}

$output .= "}\n\n";

$output .= "impl<'a> Node<'a> {\n";

foreach ($ast as $node => $structure) {
    if ($node === 'NodeId') {
        continue;
    }

    if (isset($structure['node']) && $structure['node'] === false) {
        continue;
    }

    if (isset($structure['rename'])) {
        $node = $structure['rename'];
    }

    $output .= feature_flag($structure);

    $kebab = strtolower(Str::snake($node));

    $output .= "    pub fn as_{$kebab}(self) -> Option<&'a {$node}> {\n";
    $output .= "        match &self.kind {\n";
    $output .= "            NodeKind::{$node}(node) => Some(node),\n";
    $output .= "            _ => None,\n";
    $output .= "        }\n";
    $output .= "    }\n\n";

    $output .= feature_flag($structure);

    $output .= "    pub fn is_{$kebab}(&self) -> bool {\n";
    $output .= "        matches!(&self.kind, NodeKind::{$node}(_))\n";
    $output .= "    }\n\n";
}

$output .= "pub fn name(&self) -> &'static str {\n";
$output .= "    match &self.kind {\n";

foreach ($ast as $node => $structure) {
    if ($node === 'NodeId') {
        continue;
    }

    if (isset($structure['node']) && $structure['node'] === false) {
        continue;
    }

    if (isset($structure['rename'])) {
        $node = $structure['rename'];
    }

    $output .= feature_flag($structure);
    $output .= "    NodeKind::{$node}(_) => \"{$node}\",\n";
}

$output .= "    }\n";
$output .= "}\n";

$output .= "pub fn children(&self) -> Vec<Node<'a>> {\n";
$output .= "    let mut children: Vec<Node<'a>> = Vec::new();\n";
$output .= "    match &self.kind {\n";

foreach ($ast as $node => $structure) {
    if (! isset($structure['children'])) {
        continue;
    }

    if (isset($structure['rename'])) {
        $node = $structure['rename'];
    }

    $output .= feature_flag($structure);

    $output .= "    NodeKind::{$node}(node) => {\n";

    if ($structure['children'] === ['self']) {
        $output .= "match node {\n";
        
        foreach ($structure as $sub => $subStructure) {
            if (in_array($sub, $reserved, true)) {
                continue;
            }

            if (! in_array($subStructure, ['Span'])) {
                $output .= "{$node}::{$sub}(inner) => {\n";
                if (str_starts_with($subStructure, 'Box<')) {
                    $output .= "let x = inner.as_ref();\n";
                    $output .= "children.push(x.into());\n";
                } else {
                    $output .= "    children.push(inner.into());\n";
                }
                $output .= "}\n";
            }
        }

        $output .= "_ => {},\n";
        $output .= "}\n";
    } else {
        if (collect($structure['children'])->contains(fn (string $value) => str_starts_with($value, 'self.'))) {
            $output .= "match node {\n";

            foreach ($structure as $field => $type) {
                if (in_array($field, $reserved, true)) {
                    continue;
                }

                $output .= "{$node}::{$field} { ";
                $typeFields = $ast[$node][$field];
                
                foreach ($typeFields as $typeField => $typeFieldType) {
                    $output .= "{$typeField}, ";
                }

                $output .= ".. } => {\n";

                foreach ($structure['children'] as $child) {
                    $child = substr($child, 5);
                    $child = substr($child, 0, -1);
                    $type = $typeFields[$child] ?? null;

                    if ($type) {
                        if (str_starts_with($type, 'Vec<') || $type === 'Block') {
                            $output .= "for x in &{$child} {\n";
                            $output .= "    children.push(x.into());\n";
                            $output .= "}\n";
                        } elseif (str_starts_with($type, 'Box<')) {
                            $output .= "let x = {$child}.as_ref();\n";
                            $output .= "children.push(x.into());\n";
                        } elseif (str_starts_with($type, 'CommaSeparated<')) {
                            $output .= "for x in &{$child}.inner {\n";
                            $output .= "    children.push(x.into());\n";
                            $output .= "}\n";
                        } else {
                            $output .= "let x = &{$child};\n";
                            $output .= "children.push(x.into());\n";
                        }
                    }
                }

                $output .= "}\n";
            }

            $output .= "}\n";
        } else {
            foreach ($structure['children'] as $child) {
                if (str_ends_with($child, '?')) {
                    $child = substr($child, 0, -1);
                    $output .= "if let Some(child) = &node.{$child} {\n";
                    $boxed = str_contains($structure[$child], 'Box<');

                    if ($boxed) {
                        $output .= "    let child = child.as_ref();\n";
                    }

                    $output .= "    children.push(child.into());\n";
                    $output .= "}\n";
                } elseif (str_ends_with($child, '!') && ($structure['as'] ?? null) === 'Enum') {
                } else {
                    $type = $structure[$child];

                    if (str_starts_with($type, 'Vec<') || $type === 'Block') {
                        $output .= "for x in &node.{$child} {\n";
                        $output .= "    children.push(x.into());\n";
                        $output .= "}\n";
                    } elseif (str_starts_with($type, 'Box<')) {
                        $output .= "let x = node.{$child}.as_ref();\n";
                        $output .= "children.push(x.into());\n";
                    } elseif (str_starts_with($type, 'CommaSeparated<')) {
                        $output .= "for x in &node.{$child}.inner {\n";
                        $output .= "    children.push(x.into());\n";
                        $output .= "}\n";
                    } else {
                        $output .= "let x = &node.{$child};\n";
                        $output .= "children.push(x.into());\n";
                    }
                }
            }
        }
    }

    $output .= "}\n";
}

$output .= "        _ => {}\n";
$output .= "    }\n";

$output .= "    children\n";
$output .= "}\n\n";

$output .= "pub fn as_ptr(&self) -> NonNull<()> {\n";
$output .= "match &self.kind {\n";

foreach ($ast as $node => $structure) {
    if ($node === 'NodeId') {
        continue;
    }

    if (isset($structure['node']) && $structure['node'] === false) {
        continue;
    }

    if (isset($structure['rename'])) {
        $node = $structure['rename'];
    }

    $output .= feature_flag($structure);

    $output .= "NodeKind::{$node}(node) => NonNull::from(node).cast(),\n";
}

$output .= "}\n";
$output .= "}\n";

$output .= "}\n\n";

foreach ($ast as $node => $structure) {
    if ($node === 'NodeId') {
        continue;
    }

    if (isset($structure['node']) && $structure['node'] === false) {
        continue;
    }

    if (is_string($structure)) {
        continue;
    }

    if (isset($structure['rename'])) {
        $node = $structure['rename'];
    }

    $output .= feature_flag($structure);

    $output .= "impl<'a> From<&'a {$node}> for Node<'a> {\n";
    $output .= "    fn from(node: &'a {$node}) -> Self {\n";
    $output .= "        Node::new(node.id(), NodeKind::{$node}(node), node.span())\n";
    $output .= "    }\n";
    $output .= "}\n\n";
}

file_put_contents(__DIR__ . '/../../crates/ast/src/generated.rs', $output);

echo "AST file generated.\n";
