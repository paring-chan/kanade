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
    );
}
