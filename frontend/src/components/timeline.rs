use yew::{function_component, html, use_state, Html, Callback};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

#[derive(Serialize, Deserialize, Clone)]
struct TimelineEvent {
    id: String,
    title: String,
    description: Option<String>,
    start_date: String,
    end_date: Option<String>,
    location: Option<String>,
    image_url: Option<String>,
    category: Option<String>,
}

#[function_component(Timeline)]
pub fn timeline() -> Html {
    let events = use_state(|| Vec::<TimelineEvent>::new());
    let loading = use_state(|| true);
    
    {
        let events = events.clone();
        let loading = loading.clone();
        yew::use_effect_with_deps(
            move |_| {
                let fetch_events = async move {
                    let response = Request::get("/api/events")
                        .send()
                        .await
                        .unwrap();
                    let events_data: Vec<TimelineEvent> = response.json().await.unwrap();
                    events.set(events_data);
                    loading.set(false);
                };
                wasm_bindgen_futures::spawn_local(fetch_events);
            },
            vec![],
        );
    }

    if *loading {
        return html! { <div class="text-center">Loading timeline...</div> };
    }

    html! {
        <div class="timeline-container">
            <div class="timeline">
                {events.iter().map(|event| {
                    html! {
                        <div class="timeline-event">
                            <div class="event-marker"></div>
                            <div class="event-content">
                                <h3>{&event.title}</h3>
                                <p>{&event.description.as_ref().unwrap_or(&"No description".to_string())}</p>
                                <p>{&event.start_date}</p>
                            </div>
                        </div>
                    }
                }).collect::<Html>()}
            </div>
        </div>
    }
}
