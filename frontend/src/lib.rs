use yew::{function_component, html, use_state, Html};
use yew_router::{prelude::*, Switch};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
struct Event {
    id: String,
    title: String,
    description: Option<String>,
    start_date: String,
    end_date: Option<String>,
    location: Option<String>,
    image_url: Option<String>,
    category: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Switch, Clone)]
pub enum Route {
    #[to = "/events/:id"]
    EventDetail { id: String },
    #[to = "/events"]
    Events,
    #[to = "/"]
    Home,
    #[to = "/about"]
    About,
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(routes)} />
        </BrowserRouter>
    }
}

fn routes(route: &Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::Events => html! { <Events /> },
        Route::EventDetail { id } => html! { <EventDetail id={id.clone()} /> },
        Route::About => html! { <About /> },
    }
}

#[function_component(Home)]
fn home() -> Html {
    html! {
        <div class="min-h-screen bg-base-200">
            <header class="bg-base-100 shadow">
                <div class="container mx-auto px-4 py-6">
                    <h1 class="text-3xl font-bold">Timeline Explorer</h1>
                </div>
            </header>
            <main class="container mx-auto px-4 py-8">
                <div class="hero bg-base-200 min-h-screen">
                    <div class="hero-content text-center">
                        <div class="max-w-md">
                            <h1 class="text-5xl font-bold">Welcome to Timeline Explorer</h1>
                            <p class="py-6">Explore historical events in an interactive timeline</p>
                            <a href="/events" class="btn btn-primary">View Events</a>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    }
}

#[function_component(Events)]
fn events() -> Html {
    let events = use_state(|| Vec::<Event>::new());
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
                    let events: Vec<Event> = response.json().await.unwrap();
                    events.set(events);
                    loading.set(false);
                };
                wasm_bindgen_futures::spawn_local(fetch_events);
            },
            vec![],
        );
    }

    if *loading {
        return html! { <div class="text-center">Loading...</div> };
    }

    html! {
        <div class="min-h-screen bg-base-200">
            <header class="bg-base-100 shadow">
                <div class="container mx-auto px-4 py-6">
                    <h1 class="text-3xl font-bold">Events Timeline</h1>
                </div>
            </header>
            <main class="container mx-auto px-4 py-8">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {events.iter().map(|event| {
                        html! {
                            <div class="card bg-base-100 shadow-xl">
                                <div class="card-body">
                                    <h2 class="card-title">{&event.title}</h2>
                                    <p>{&event.description.as_ref().unwrap_or(&"No description".to_string())}</p>
                                    <div class="card-actions justify-end">
                                        <a href={format!("/events/{}", event.id)} class="btn btn-primary">View Details</a>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()}
                </div>
            </main>
        </div>
    }
}

#[function_component(EventDetail)]
fn event_detail(props: &EventDetailProps) -> Html {
    let event = use_state(|| Option::<Event>::None);
    let loading = use_state(|| true);
    
    {
        let event = event.clone();
        let loading = loading.clone();
        let id = props.id.clone();
        yew::use_effect_with_deps(
            move |_| {
                let fetch_event = async move {
                    let response = Request::get(&format!("/api/events/{}", id))
                        .send()
                        .await
                        .unwrap();
                    let event_data: Event = response.json().await.unwrap();
                    event.set(Some(event_data));
                    loading.set(false);
                };
                wasm_bindgen_futures::spawn_local(fetch_event);
            },
            vec![id],
        );
    }

    if *loading {
        return html! { <div class="text-center">Loading...</div> };
    }

    let event_data = event.as_ref().unwrap();
    
    html! {
        <div class="min-h-screen bg-base-200">
            <header class="bg-base-100 shadow">
                <div class="container mx-auto px-4 py-6">
                    <h1 class="text-3xl font-bold">Event Details</h1>
                </div>
            </header>
            <main class="container mx-auto px-4 py-8">
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title text-2xl">{&event_data.title}</h2>
                        <p>{&event_data.description.as_ref().unwrap_or(&"No description".to_string())}</p>
                        <div class="mt-4">
                            <p><strong>Start Date:</strong> {&event_data.start_date}</p>
                            {if let Some(end_date) = &event_data.end_date {
                                html! { <p><strong>End Date:</strong> {end_date}</p> }
                            } else {
                                html! {}
                            }}
                            {if let Some(location) = &event_data.location {
                                html! { <p><strong>Location:</strong> {location}</p> }
                            } else {
                                html! {}
                            }}
                            {if let Some(category) = &event_data.category {
                                html! { <p><strong>Category:</strong> {category}</p> }
                            } else {
                                html! {}
                            }}
                        </div>
                        {if let Some(image_url) = &event_data.image_url {
                            html! { <img src={image_url.clone()} alt={&event_data.title} class="mt-4 rounded-lg" /> }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>
            </main>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct EventDetailProps {
    id: String,
}

#[function_component(About)]
fn about() -> Html {
    html! {
        <div class="min-h-screen bg-base-200">
            <header class="bg-base-100 shadow">
                <div class="container mx-auto px-4 py-6">
                    <h1 class="text-3xl font-bold">About Timeline Explorer</h1>
                </div>
            </header>
            <main class="container mx-auto px-4 py-8">
                <div class="prose max-w-none">
                    <p>This timeline application allows you to explore historical events in an interactive way.</p>
                    <p>Features include:</p>
                    <ul>
                        <li>Zoomable and pannable timeline</li>
                        <li>Event details with images</li>
                        <li>Search and filtering capabilities</li>
                        <li>Responsive design</li>
                    </ul>
                </div>
            </main>
        </div>
    }
}
