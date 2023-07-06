use leptos::*;
use std::sync::Arc;
use unterstutzen::{Calendar, Configuration};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    let values = create_resource(cx, || (), |_| async move { load_events().await.unwrap() });
    view! { cx,

        <button
            on:click=move |_| {
                set_count.update(|n| *n += 1);
            }
        >
            "Click for more shots of espresso: "
            {move || count.get()}
        </button>


        <p>"Events:"</p>
        {move || match values.read(cx) {
                None => view! { cx, <p>"Loading events..."</p> }.into_view(cx),
                Some(data) => view! { cx,
                        <ul>
                            {data.into_iter()
                            .map(|n| view! { cx, <li>{n}</li>})
                            .collect::<Vec<_>>()}
                        </ul>
                }.into_view(cx)
        }}
    }
}

async fn load_events() -> anyhow::Result<Vec<String>> {
    web_sys::console::log_1(&"hi 0".into());
    let config = Arc::new(Configuration::load()?);
    web_sys::console::log_1(&"hi 1".into());
    let calendar = Calendar::from(&config);
    web_sys::console::log_1(&"hi 2".into());
    let events = calendar.events().await?;
    web_sys::console::log_1(&"hi 3".into());
    Ok(
        events
            .items
            .iter()
            .map(|e| format!("{e:?}"))
            .collect()
    )
}
