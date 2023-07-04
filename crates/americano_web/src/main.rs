use leptos::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    view! { cx,

        <button
            on:click=move |_| {
                set_count.update(|n| *n += 1);
            }
        >
            "Click for more shots of espresso: "
            {move || count.get()}
        </button>
    }
}

fn main() {
    leptos::mount_to_body(|cx| view! { cx, <App/>  })
}
