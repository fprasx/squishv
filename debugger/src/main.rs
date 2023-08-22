#![allow(non_snake_case)]
use dioxus::prelude::*;
use log::{debug, Level};
use riscv::{
    executor::{Diff, ExecResult, Executor, RegisterSnapshot, Update, REGISTERS},
    parse::Program,
};

fn main() {
    console_log::init_with_level(Level::Debug).expect("initializing logger should not fail");
    // launch the web app
    dioxus_web::launch(App);
}
// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    use_shared_state_provider(cx, || Executor::new("".parse::<Program>().unwrap()));
    let executor = use_shared_state::<Executor>(cx).expect("program context was provided");
    let diff = use_state::<Option<ExecResult<Update>>>(cx, || None);
    let forward = move |_| {
        let mut guard = executor.write();
        diff.set(Some(guard.execute()));
    };

    let run = move |_| {
        let mut guard = executor.write();
        // Don't use while let as we want to set diff to the error value when it
        // happens, instead of just ignoring it and breaking
        loop {
            let update = guard.execute();
            let exit = update.is_err();
            diff.set(Some(update));
            if exit {
                break;
            }
        }
    };

    let style = "bg-red-400 p-2 m-2";

    cx.render(rsx! {
        button {
            class: style,
            onclick: |_| todo!(),
            "<< back"
        }
        button {
            class: style,
            onclick: forward,
            "forward >>"
        }
        button {
            onclick: run,
            "run"
        }
        div {
            class: "flex flex-row",
            div {
                class: "bg-yellow-400",
                CodeInput {}
            }
            div {
                class: "bg-blue-400",
                Registers {
                    regs: &executor.read().regfile,
                    diff: diff.get().as_ref(),
                }
            }
        }
    })
}

#[derive(Props)]
struct RegisterProps<'a> {
    regs: &'a RegisterSnapshot,
    #[props(!optional)]
    diff: Option<&'a ExecResult<Update>>,
}

fn Registers<'a>(cx: Scope<'a, RegisterProps<'a>>) -> Element {
    let status = match cx.props.diff {
        Some(Ok(Update {
            nextpc,
            diff: Some(Diff::Register { reg, val }),
        })) => format!("pc -> {nextpc},  {reg} -> {val}"),
        Some(Ok(Update {
            nextpc,
            diff: Some(Diff::Memory { addr, val, op }),
        })) => format!("pc -> {nextpc},  {addr} -> {val} via {op}"),
        Some(Ok(Update { nextpc, diff: None })) => format!("pc -> {nextpc}"),
        Some(Err(e)) => format!("error executing: {e}"),
        None => "execution complete".to_string(),
    };

    let changed = if let Some(Ok(Update { diff: Some(Diff::Register { reg, .. }), .. })) = cx.props.diff {
        Some(reg)
    } else {
        None
    };

    cx.render(rsx! {
        div {
            div {
                status
            }
            div {
                class: "grid grid-cols-4 grid-flow-row gap-4",
                for reg in REGISTERS {
                    div {
                        class: format_args!(
                            "{}",
                            if Some(&reg) == changed { "bg-red-400" } else { "" }
                        ),
                        "{reg}: {cx.props.regs[reg]}"
                    }
                }
            }
        }
    })
}

fn CodeInput(cx: Scope<'_>) -> Element {
    let exec = use_shared_state::<Executor>(cx).expect("executor context was provided");
    let error = use_state::<Option<anyhow::Error>>(cx, || None);
    let lines = use_state::<usize>(cx, || 1);
    cx.render(rsx! {
        div {
            class: "p-2",
            textarea {
                oninput: move |e| {
                    let text = e.value.clone();
                    let parsed = text.as_str().parse::<Program>();
                    match parsed {
                        Ok(p) => {
                            *exec.write() = Executor::new(p);
                            error.set(None);
                        }
                        Err(e) => {
                            error.set(Some(e))
                        }
                    }
                    lines.set(text.split('\n').count());
                },
                cols: 20,
                rows: 20, 
                spellcheck: false,
                placeholder: "assembly . . .",
                class: format_args!("{border_color} p-2 placeholder:text-slate-400 focus:outline-blue-400",
                    border_color = if error.is_some() {
                        "border-2 border-red-400"
                    } else {
                        "border-2 border-green-400"
                    },
                ),
            }
            div {
                class: "text-red-400 p-2",
                if let Some(error) = error.get().as_ref() {
                    rsx! {
                        ul {
                            for part in error.chain() {
                                li {
                                    class: "text-ellipsis overflow-hidden",
                                    part.to_string()
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
