let lint = ctx.pipeline("lint");

if ctx.event == "push" && ctx.branch == "main" {
    let cargo_check = lint.job("cargo-check", #{ name: "Cargo Check" });


    cargo_check.step(#{
        name: "Set up repository",
        command: #"
        git clone $CLONE_URL tree
        cd tree
        echo $PWD > /workspace/cwd

        mise install
        "#
    });
    cargo_check.step(#{
        name: "Check",
        command: #"

        mise x -- cargo check --locked
        "#
    });

    let frontend_check = lint.job("frontend-check", #{ name: "Frontend Check" });
    frontend_check.step(#{
        name: "Set up repository",
        command: #"
        git clone $CLONE_URL tree
        cd tree/apps/frontend
        echo $PWD > /workspace/cwd

        mise install
        "#
    });
    frontend_check.step(#{
        name: "Install dependencies",
        command: "mise x -- bun i"
    });
    frontend_check.step(#{
        name: "Check",
        command: "mise x -- bun check"
    });
}
