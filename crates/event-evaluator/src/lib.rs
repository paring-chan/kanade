use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rhai::serde::to_dynamic;
use rhai::{Engine, Scope, exported_module};

use crate::types::EvalContext;

mod types;

pub fn evaluate(script: &str) {
    let mut engine = Engine::new();
    let module = exported_module!(types::kanade_rhai_module);
    engine.register_global_module(module.into());

    let mut scope = Scope::new();
    scope.push(
        "ctx",
        Rc::new(RefCell::new(EvalContext {
            event: "push".into(),
            branch: "main".into(),
            tag: None,
            args: to_dynamic(serde_json::json!({
                "wow": true
            }))
            .unwrap(),
            pipelines: HashMap::new(),
            default_image: "oven/bun:latest".into(),
            default_shell: "/bin/sh".into(),
        })),
    );

    engine.run_with_scope(&mut scope, script).unwrap();
}
