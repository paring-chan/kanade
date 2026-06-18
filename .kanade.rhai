let lint = ctx.pipeline("lint");

if ctx.event == "push" && ctx.branch == "main" {
    let cargo_check = lint.job("cargo-check", #{ name: "Cargo Check" });

    cargo_check.step(#{
        name: "Set up repository",
        command: #"
        set -ex
        git clone $CLONE_URL tree
        cd tree
        echo $PWD > /workspace/cwd

        mise install
        "#
    });

    cargo_check.step(#{
        name: "Check",
        command: #"
        set -ex

        mise x -- cargo check --locked
        "#
    });
}
