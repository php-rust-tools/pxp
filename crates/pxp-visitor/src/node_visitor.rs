use pxp_ast::{Node, Statement};

pub trait NodeVisitor {
    fn visit_ast(&mut self, ast: &[Statement]) {
        for node in ast {
            let result = self.visit_node(node);

            if result == NodeVisitorResult::Stop {
                break;
            }
        }
    }

    fn visit_node(&mut self, node: &dyn Node) -> NodeVisitorResult {
        let result = self.visit(node);

        if result != NodeVisitorResult::SkipChildren {
            for child in node.children() {
                let inner_result = self.visit_node(child);

                if inner_result == NodeVisitorResult::Stop {
                    return NodeVisitorResult::Stop;
                }
            }
        }

        result
    }

    fn visit(&mut self, node: &dyn Node) -> NodeVisitorResult;
}

#[derive(PartialEq, Eq)]
pub enum NodeVisitorResult {
    Continue,
    SkipChildren,
    Stop,
}