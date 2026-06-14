use std::collections::HashMap;

use event_evaluator::script::EvalContext;
use rhai::serde::to_dynamic;

fn main() {
    event_evaluator::evaluate(
        r#"
        let build = ctx.pipeline("build");

        if ctx.event == "push" {
            build.job("hello").step(#{
                name: "clone",
                command: "git clone $CLONE_URL",
            });
        }
        "#,
        EvalContext {
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
        },
    )
    .unwrap();
}
