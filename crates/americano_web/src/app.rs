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
    eprintln!("hi 0");
    let config = Arc::new(Configuration::load()?);
    eprintln!("hi 1");
    let calendar = Calendar::from(&config);
    eprintln!("hi 2");
    let events = calendar.events().await?;
    eprintln!("hi 3");
    Ok(
        events
            .items
            .iter()
            .map(|e| format!("{e:?}"))
            .collect()
    )
}
