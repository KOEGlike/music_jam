use leptos::logging::log;
use leptos::*;
use leptonic::components::icon::*;
use leptonic::components::input::TextInput;



#[component]
pub fn JoinIsland() -> impl IntoView {
    let (feedback, set_feedback)=create_signal(String::from("PLEASE TYPE IN A JAM CODE"));
    let (jam_code, set_jam_code) = create_signal(String::new());
    let on_click =move|_|{ log!("Joining jam code: {}", jam_code.get());};

    view! {
        <div class="standard_island">
            {feedback}
            <TextInput set=set_jam_code get=jam_code placeholder="ex. 786908"/>
            <button on:click=on_click class="button">"Join"</button>
        </div>
    }
}

