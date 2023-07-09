use leptos::*;
use leptos_meta::*;
use leptos_router::*;
//use std::sync::Arc;
//use unterstutzen::{Calendar, Configuration};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);
    let values = create_local_resource(cx, || (), |_| async move { load_events().await.unwrap() });

    view! { cx,

        <h1>"Americano ☕️"</h1>

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

            }
        }
    }
}

#[server(LoadEvents, "/api", "GetJson")]
async fn load_events() -> Result<Vec<String>, ServerFnError> {
    let config = Arc::new(Configuration::load()?);
    let calendar = Calendar::from(&config);
    let events = calendar.events().await?;
    Ok(events.items.iter().map(|e| format!("{e:?}")).collect())
}
